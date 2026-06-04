#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JassType {
    Int,
    Real,
    Bool,
    Str,
    Handle,
    Code,
    Void,
}

pub fn parse_signature(sig: &str) -> Result<(Vec<JassType>, JassType), String> {
    let bytes = sig.as_bytes();

    if bytes.first() != Some(&b'(') {
        return Err(format!("signature missing '(': {sig}"));
    }

    let close = bytes
        .iter()
        .position(|&c| c == b')')
        .ok_or_else(|| format!("signature missing ')': {sig}"))?;

    let mut args = Vec::new();
    let mut cursor = 1;
    while cursor < close {
        let (ty, consumed) = parse_one(&sig[cursor..close])?;
        args.push(ty);
        cursor += consumed;
    }

    let ret_str = &sig[close + 1..];
    if ret_str.is_empty() {
        return Err(format!("signature missing return type: {sig}"));
    }

    let (ret, _) = parse_one(ret_str)?;
    Ok((args, ret))
}

fn parse_one(s: &str) -> Result<(JassType, usize), String> {
    let c = s.chars().next().ok_or_else(|| "empty type".to_string())?;

    match c {
        'I' => Ok((JassType::Int, 1)),
        'R' => Ok((JassType::Real, 1)),
        'B' => Ok((JassType::Bool, 1)),
        'S' => Ok((JassType::Str, 1)),
        'C' => Ok((JassType::Code, 1)),
        'V' => Ok((JassType::Void, 1)),
        'H' => {
            let semi = s
                .bytes()
                .position(|b| b == b';')
                .ok_or_else(|| format!("unterminated handle: {s}"))?;
            Ok((JassType::Handle, semi + 1))
        }
        other => Err(format!("unknown type char '{other}' in: {s}")),
    }
}
