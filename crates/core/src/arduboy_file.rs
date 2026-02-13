//! `.arduboy` file parser.
//!
//! An `.arduboy` file is a ZIP archive containing:
//! - `info.json` — metadata (title, author, description)
//! - `*.hex` — Intel HEX game binary
//! - `*-fx.bin` or `*.bin` — optional FX flash data
//!
//! This module provides a minimal ZIP reader (stored + deflate) to extract
//! these files without any external dependencies.

use std::collections::HashMap;

/// Parsed contents of an .arduboy file.
#[derive(Debug, Default)]
pub struct ArduboyFile {
    /// Game title from info.json (if present).
    pub title: String,
    /// Author from info.json (if present).
    pub author: String,
    /// Intel HEX data as a string.
    pub hex: Option<String>,
    /// FX flash binary data.
    pub fx_data: Option<Vec<u8>>,
    /// All files in the archive: name → data.
    pub files: HashMap<String, Vec<u8>>,
}

/// Parse a .arduboy (ZIP) file from raw bytes.
pub fn parse_arduboy(data: &[u8]) -> Result<ArduboyFile, String> {
    let files = read_zip(data)?;
    let mut result = ArduboyFile::default();
    result.files = files.clone();

    // Find hex file
    for (name, content) in &files {
        let lower = name.to_lowercase();
        if lower.ends_with(".hex") {
            result.hex = Some(String::from_utf8_lossy(content).into_owned());
        }
    }

    // Find FX data: prefer *-fx.bin, then *.bin (but not info.*)
    for (name, content) in &files {
        let lower = name.to_lowercase();
        if lower.ends_with("-fx.bin") {
            result.fx_data = Some(content.clone());
            break;
        }
    }
    if result.fx_data.is_none() {
        for (name, content) in &files {
            let lower = name.to_lowercase();
            if lower.ends_with(".bin") && !lower.contains("info") {
                result.fx_data = Some(content.clone());
                break;
            }
        }
    }

    // Parse info.json (simple key extraction, no full JSON parser)
    if let Some(info_data) = files.get("info.json").or_else(|| files.get("INFO.JSON")) {
        let info_str = String::from_utf8_lossy(info_data);
        result.title = extract_json_string(&info_str, "title")
            .or_else(|| extract_json_string(&info_str, "name"))
            .unwrap_or_default();
        result.author = extract_json_string(&info_str, "author")
            .or_else(|| extract_json_string(&info_str, "developer"))
            .unwrap_or_default();
    }

    if result.hex.is_none() {
        return Err("No .hex file found in .arduboy archive".into());
    }

    Ok(result)
}

/// Simple JSON string value extractor (no full parser).
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = json.find(&pattern)?;
    let rest = &json[idx + pattern.len()..];
    // Skip whitespace and colon
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

// ─── Minimal ZIP Reader ─────────────────────────────────────────────────────

fn read_zip(data: &[u8]) -> Result<HashMap<String, Vec<u8>>, String> {
    let mut files = HashMap::new();
    let mut pos = 0;

    while pos + 4 <= data.len() {
        let sig = u32_le(data, pos);
        if sig != 0x04034b50 { break; } // Local file header

        if pos + 30 > data.len() { break; }
        let method = u16_le(data, pos + 8);
        let comp_size = u32_le(data, pos + 18) as usize;
        let uncomp_size = u32_le(data, pos + 22) as usize;
        let name_len = u16_le(data, pos + 26) as usize;
        let extra_len = u16_le(data, pos + 28) as usize;

        let name_start = pos + 30;
        if name_start + name_len > data.len() { break; }
        let name = String::from_utf8_lossy(&data[name_start..name_start + name_len]).into_owned();

        let data_start = name_start + name_len + extra_len;
        if data_start + comp_size > data.len() { break; }
        let compressed = &data[data_start..data_start + comp_size];

        let file_data = match method {
            0 => compressed.to_vec(), // Stored
            8 => { // Deflate
                inflate(compressed, uncomp_size)
                    .map_err(|e| format!("Inflate error for {}: {}", name, e))?
            }
            _ => {
                // Skip unsupported compression methods
                pos = data_start + comp_size;
                continue;
            }
        };

        // Skip directories
        if !name.ends_with('/') {
            // Strip directory prefix for simpler lookup
            let simple_name = name.rsplit('/').next().unwrap_or(&name).to_string();
            files.insert(simple_name, file_data.clone());
            // Also insert full path
            if name.contains('/') {
                files.insert(name.clone(), file_data);
            }
        }

        pos = data_start + comp_size;
    }

    if files.is_empty() {
        return Err("No files found in ZIP archive".into());
    }
    Ok(files)
}

fn u16_le(data: &[u8], pos: usize) -> u16 {
    (data[pos] as u16) | ((data[pos + 1] as u16) << 8)
}
fn u32_le(data: &[u8], pos: usize) -> u32 {
    (data[pos] as u32) | ((data[pos + 1] as u32) << 8)
    | ((data[pos + 2] as u32) << 16) | ((data[pos + 3] as u32) << 24)
}

// ─── Minimal Inflate (RFC 1951) ─────────────────────────────────────────────

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader { data, pos: 0, bit: 0 }
    }
    fn read_bits(&mut self, n: u8) -> Result<u32, String> {
        let mut val = 0u32;
        for i in 0..n {
            if self.pos >= self.data.len() { return Err("Unexpected end of data".into()); }
            let b = ((self.data[self.pos] >> self.bit) & 1) as u32;
            val |= b << i;
            self.bit += 1;
            if self.bit >= 8 { self.bit = 0; self.pos += 1; }
        }
        Ok(val)
    }
    fn align(&mut self) {
        if self.bit != 0 { self.bit = 0; self.pos += 1; }
    }
}

fn inflate(data: &[u8], expected_size: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(expected_size);
    let mut br = BitReader::new(data);
    loop {
        let bfinal = br.read_bits(1)?;
        let btype = br.read_bits(2)?;
        match btype {
            0 => inflate_stored(&mut br, &mut out)?,
            1 => inflate_fixed(&mut br, &mut out)?,
            2 => inflate_dynamic(&mut br, &mut out)?,
            _ => return Err("Invalid block type 3".into()),
        }
        if bfinal == 1 { break; }
    }
    Ok(out)
}

fn inflate_stored(br: &mut BitReader, out: &mut Vec<u8>) -> Result<(), String> {
    br.align();
    if br.pos + 4 > br.data.len() { return Err("Stored block truncated".into()); }
    let len = (br.data[br.pos] as u16) | ((br.data[br.pos + 1] as u16) << 8);
    br.pos += 4; // skip LEN and NLEN
    for _ in 0..len {
        if br.pos >= br.data.len() { return Err("Stored data truncated".into()); }
        out.push(br.data[br.pos]);
        br.pos += 1;
    }
    Ok(())
}

// Fixed Huffman code lengths (RFC 1951 §3.2.6)
fn inflate_fixed(br: &mut BitReader, out: &mut Vec<u8>) -> Result<(), String> {
    // Build fixed literal/length tree
    let mut lengths = [0u8; 288];
    for i in 0..=143 { lengths[i] = 8; }
    for i in 144..=255 { lengths[i] = 9; }
    for i in 256..=279 { lengths[i] = 7; }
    for i in 280..=287 { lengths[i] = 8; }
    let lit_tree = build_huffman(&lengths)?;

    let dist_lengths = [5u8; 32];
    let dist_tree = build_huffman(&dist_lengths)?;

    inflate_block(br, out, &lit_tree, &dist_tree)
}

fn inflate_dynamic(br: &mut BitReader, out: &mut Vec<u8>) -> Result<(), String> {
    let hlit = br.read_bits(5)? as usize + 257;
    let hdist = br.read_bits(5)? as usize + 1;
    let hclen = br.read_bits(4)? as usize + 4;

    const ORDER: [usize; 19] = [16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15];
    let mut cl_lengths = [0u8; 19];
    for i in 0..hclen {
        cl_lengths[ORDER[i]] = br.read_bits(3)? as u8;
    }
    let cl_tree = build_huffman(&cl_lengths)?;

    let mut lengths = vec![0u8; hlit + hdist];
    let mut i = 0;
    while i < lengths.len() {
        let sym = decode_symbol(br, &cl_tree)?;
        match sym {
            0..=15 => { lengths[i] = sym as u8; i += 1; }
            16 => {
                let rep = br.read_bits(2)? as usize + 3;
                let val = if i > 0 { lengths[i - 1] } else { 0 };
                for _ in 0..rep { if i < lengths.len() { lengths[i] = val; i += 1; } }
            }
            17 => {
                let rep = br.read_bits(3)? as usize + 3;
                for _ in 0..rep { if i < lengths.len() { lengths[i] = 0; i += 1; } }
            }
            18 => {
                let rep = br.read_bits(7)? as usize + 11;
                for _ in 0..rep { if i < lengths.len() { lengths[i] = 0; i += 1; } }
            }
            _ => return Err(format!("Invalid code length symbol {}", sym)),
        }
    }

    let lit_tree = build_huffman(&lengths[..hlit])?;
    let dist_tree = build_huffman(&lengths[hlit..])?;
    inflate_block(br, out, &lit_tree, &dist_tree)
}

fn inflate_block(br: &mut BitReader, out: &mut Vec<u8>,
    lit_tree: &HuffTree, dist_tree: &HuffTree) -> Result<(), String>
{
    static LEN_BASE: [u16; 29] = [3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,
        51,59,67,83,99,115,131,163,195,227,258];
    static LEN_EXTRA: [u8; 29] = [0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0];
    static DIST_BASE: [u16; 30] = [1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,
        513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577];
    static DIST_EXTRA: [u8; 30] = [0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13];

    loop {
        let sym = decode_symbol(br, lit_tree)?;
        if sym < 256 {
            out.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            let li = (sym - 257) as usize;
            if li >= LEN_BASE.len() { return Err(format!("Invalid length code {}", sym)); }
            let length = LEN_BASE[li] as usize + br.read_bits(LEN_EXTRA[li])? as usize;
            let di = decode_symbol(br, dist_tree)? as usize;
            if di >= DIST_BASE.len() { return Err(format!("Invalid dist code {}", di)); }
            let dist = DIST_BASE[di] as usize + br.read_bits(DIST_EXTRA[di])? as usize;
            for _ in 0..length {
                let pos = out.len().wrapping_sub(dist);
                let b = if pos < out.len() { out[pos] } else { 0 };
                out.push(b);
            }
        }
    }
}

// ─── Huffman Tree ───────────────────────────────────────────────────────────

struct HuffTree {
    // Lookup table: [bits consumed, symbol] indexed by reversed code
    table: Vec<(u8, u16)>,
    max_bits: u8,
}

fn build_huffman(lengths: &[u8]) -> Result<HuffTree, String> {
    let max_bits = *lengths.iter().max().unwrap_or(&0);
    if max_bits == 0 {
        return Ok(HuffTree { table: vec![(1, 0); 2], max_bits: 1 });
    }
    let max_bits = max_bits.min(15);
    let table_size = 1usize << max_bits;
    let mut table = vec![(0u8, 0u16); table_size];

    // Count per bit length
    let mut bl_count = [0u32; 16];
    for &l in lengths { if l > 0 { bl_count[l as usize] += 1; } }

    // Compute first code per length
    let mut next_code = [0u32; 16];
    let mut code = 0u32;
    for bits in 1..=max_bits as usize {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }

    // Fill table
    for (sym, &len) in lengths.iter().enumerate() {
        if len == 0 { continue; }
        let len = len as usize;
        let code = next_code[len];
        next_code[len] += 1;
        // Reverse bits
        let mut rev = 0u32;
        for i in 0..len { rev |= ((code >> i) & 1) << (len - 1 - i); }
        // Fill all table entries with this prefix
        let step = 1 << len;
        let mut idx = rev as usize;
        while idx < table_size {
            table[idx] = (len as u8, sym as u16);
            idx += step;
        }
    }

    Ok(HuffTree { table, max_bits })
}

fn decode_symbol(br: &mut BitReader, tree: &HuffTree) -> Result<u16, String> {
    // Peek max_bits
    let mut val = 0u32;
    let mut bits_read = 0u8;
    let save_pos = br.pos;
    let save_bit = br.bit;
    for i in 0..tree.max_bits {
        if br.pos >= br.data.len() { break; }
        let b = ((br.data[br.pos] >> br.bit) & 1) as u32;
        val |= b << i;
        bits_read += 1;
        br.bit += 1;
        if br.bit >= 8 { br.bit = 0; br.pos += 1; }
    }
    let entry = tree.table.get(val as usize).copied().unwrap_or((bits_read, 0));
    let (code_len, sym) = entry;
    if code_len == 0 { return Err("Invalid Huffman code".into()); }
    // Rewind to save point and advance exactly code_len bits
    br.pos = save_pos;
    br.bit = save_bit;
    br.read_bits(code_len)?;
    Ok(sym)
}
