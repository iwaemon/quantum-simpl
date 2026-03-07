pub mod ast;

use ast::*;

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(input: &str) -> Result<ModelDef, ParseError> {
    let mut lattice: Option<LatticeDef> = None;
    let mut sum_blocks: Vec<SumBlock> = Vec::new();
    let mut params: Vec<(String, f64)> = Vec::new();

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line.starts_with("lattice") {
            lattice = Some(parse_lattice(line)?);
            i += 1;
        } else if line.starts_with("sum") {
            let (block, next_i) = parse_sum_block(&lines, i)?;
            sum_blocks.push(block);
            i = next_i;
        } else if line.starts_with("params:") {
            let (p, next_i) = parse_params(&lines, i + 1)?;
            params = p;
            i = next_i;
        } else {
            return Err(ParseError(format!("Unexpected line: '{}'", line)));
        }
    }

    let lattice = lattice.ok_or_else(|| ParseError("Missing lattice declaration".to_string()))?;

    Ok(ModelDef { lattice, sum_blocks, params })
}

fn parse_lattice(line: &str) -> Result<LatticeDef, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return Err(ParseError(format!("Invalid lattice line: '{}'", line)));
    }

    let dimension = parts[1].to_string();
    let mut num_sites = 0usize;
    let mut pbc = false;

    for part in &parts[2..] {
        if let Some(val) = part.strip_prefix("sites=") {
            num_sites = val.parse().map_err(|_| ParseError(format!("Invalid sites: {}", val)))?;
        } else if let Some(val) = part.strip_prefix("pbc=") {
            pbc = val.parse().map_err(|_| ParseError(format!("Invalid pbc: {}", val)))?;
        }
    }

    if num_sites == 0 {
        return Err(ParseError("sites must be > 0".to_string()));
    }

    Ok(LatticeDef { dimension, num_sites, pbc })
}

fn parse_sum_block(lines: &[&str], start: usize) -> Result<(SumBlock, usize), ParseError> {
    let header = lines[start].trim();
    let header = header.strip_prefix("sum").ok_or_else(|| ParseError("Expected 'sum'".to_string()))?.trim();
    let header = header.strip_suffix(':').ok_or_else(|| ParseError("Expected ':' after sum range".to_string()))?.trim();

    let eq_pos = header.find('=').ok_or_else(|| ParseError("Expected '=' in sum".to_string()))?;
    let var = header[..eq_pos].trim().to_string();
    let range_str = &header[eq_pos + 1..];
    let dots = range_str.find("..").ok_or_else(|| ParseError("Expected '..' in range".to_string()))?;
    let range_start: usize = range_str[..dots].trim().parse()
        .map_err(|_| ParseError("Invalid range start".to_string()))?;
    let range_end: usize = range_str[dots + 2..].trim().parse()
        .map_err(|_| ParseError("Invalid range end".to_string()))?;

    let mut expressions = Vec::new();
    let mut i = start + 1;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            break;
        }

        // Check if the line is indented (part of this sum block)
        if !line.starts_with(' ') && !line.starts_with('\t') {
            break;
        }

        if trimmed.starts_with("params:") || trimmed.starts_with("sum") || trimmed.starts_with("lattice") {
            break;
        }

        expressions.push(parse_expression(trimmed)?);
        i += 1;
    }

    Ok((SumBlock { var, range_start, range_end, expressions }, i))
}

fn parse_params(lines: &[&str], start: usize) -> Result<(Vec<(String, f64)>, usize), ParseError> {
    let mut params = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }
        if !lines[i].starts_with(' ') && !lines[i].starts_with('\t') {
            break;
        }

        // Parse "name = value"
        let eq_pos = line.find('=').ok_or_else(|| ParseError(format!("Expected '=' in param: {}", line)))?;
        let name = line[..eq_pos].trim().to_string();
        let value: f64 = line[eq_pos + 1..].trim().parse()
            .map_err(|_| ParseError(format!("Invalid param value: {}", line)))?;
        params.push((name, value));
        i += 1;
    }

    Ok((params, i))
}

fn parse_expression(line: &str) -> Result<Expression, ParseError> {
    // Strip h.c. suffix
    let (line, hc) = if line.ends_with("+ h.c.") {
        (line[..line.len() - 6].trim(), true)
    } else {
        (line, false)
    };

    let tokens = tokenize(line)?;
    build_expression(&tokens, hc)
}

fn tokenize(line: &str) -> Result<Vec<String>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                chars.next();
            }
            '*' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                tokens.push("*".to_string());
                chars.next();
            }
            '(' => {
                current.push(ch);
                chars.next();
                while let Some(&c) = chars.peek() {
                    current.push(c);
                    chars.next();
                    if c == ')' { break; }
                }
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn build_expression(tokens: &[String], hc: bool) -> Result<Expression, ParseError> {
    let mut coeff_parts: Vec<String> = Vec::new();
    let mut operators = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        if token == "*" {
            i += 1;
            continue;
        }

        if token.contains('(') {
            // This is an operator; parse it and all subsequent operators
            operators.push(parse_single_operator(token)?);
            i += 1;
            while i < tokens.len() {
                if tokens[i] == "*" { i += 1; continue; }
                if tokens[i].contains('(') {
                    operators.push(parse_single_operator(&tokens[i])?);
                }
                i += 1;
            }
            break;
        } else {
            coeff_parts.push(token.clone());
            i += 1;
        }
    }

    let coeff = parse_coeff_from_strings(&coeff_parts)?;

    Ok(Expression { coeff, operators, hermitian_conjugate: hc })
}

fn parse_coeff_from_strings(parts: &[String]) -> Result<CoeffExpr, ParseError> {
    if parts.is_empty() {
        return Ok(CoeffExpr::Literal(1.0));
    }

    let mut result: Option<CoeffExpr> = None;

    for part in parts {
        let part = part.trim();
        let expr = if part.starts_with('-') {
            let inner = &part[1..];
            if inner.is_empty() {
                CoeffExpr::Neg(Box::new(CoeffExpr::Literal(1.0)))
            } else if let Ok(v) = inner.parse::<f64>() {
                CoeffExpr::Literal(-v)
            } else {
                CoeffExpr::Neg(Box::new(CoeffExpr::Param(inner.to_string())))
            }
        } else if let Ok(v) = part.parse::<f64>() {
            CoeffExpr::Literal(v)
        } else {
            CoeffExpr::Param(part.to_string())
        };

        result = Some(match result {
            None => expr,
            Some(prev) => CoeffExpr::Mul(Box::new(prev), Box::new(expr)),
        });
    }

    Ok(result.unwrap())
}

fn parse_single_operator(token: &str) -> Result<OpExpr, ParseError> {
    let paren_start = token.find('(')
        .ok_or_else(|| ParseError(format!("Expected '(' in operator: {}", token)))?;
    let paren_end = token.find(')')
        .ok_or_else(|| ParseError(format!("Expected ')' in operator: {}", token)))?;

    let name = &token[..paren_start];
    let args_str = &token[paren_start + 1..paren_end];
    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

    match name {
        "c†" => {
            if args.len() != 2 {
                return Err(ParseError(format!("c† expects 2 args, got {}", args.len())));
            }
            Ok(OpExpr::FermionCreate(parse_index(args[0])?, parse_spin(args[1])?))
        }
        "c" => {
            if args.len() != 2 {
                return Err(ParseError(format!("c expects 2 args, got {}", args.len())));
            }
            Ok(OpExpr::FermionAnnihilate(parse_index(args[0])?, parse_spin(args[1])?))
        }
        "n" => {
            if args.len() != 2 {
                return Err(ParseError(format!("n expects 2 args, got {}", args.len())));
            }
            Ok(OpExpr::Number(parse_index(args[0])?, parse_spin(args[1])?))
        }
        "Sp" => Ok(OpExpr::SpinPlus(parse_index(args[0])?)),
        "Sm" => Ok(OpExpr::SpinMinus(parse_index(args[0])?)),
        "Sz" => Ok(OpExpr::SpinZ(parse_index(args[0])?)),
        _ => Err(ParseError(format!("Unknown operator: {}", name))),
    }
}

fn parse_index(s: &str) -> Result<IndexExpr, ParseError> {
    let s = s.trim();

    if let Ok(n) = s.parse::<usize>() {
        return Ok(IndexExpr::Literal(n));
    }

    if let Some(pos) = s.find('+') {
        let var = s[..pos].trim().to_string();
        let offset: usize = s[pos + 1..].trim().parse()
            .map_err(|_| ParseError(format!("Invalid index offset: {}", s)))?;
        return Ok(IndexExpr::VarPlus(var, offset));
    }

    // Check for minus, but only if it's not the start (that would be a negative literal)
    if let Some(pos) = s[1..].find('-') {
        let pos = pos + 1; // adjust for the slice offset
        let var = s[..pos].trim().to_string();
        let offset: usize = s[pos + 1..].trim().parse()
            .map_err(|_| ParseError(format!("Invalid index offset: {}", s)))?;
        return Ok(IndexExpr::VarMinus(var, offset));
    }

    Ok(IndexExpr::Var(s.to_string()))
}

fn parse_spin(s: &str) -> Result<SpinExpr, ParseError> {
    match s.trim() {
        "up" => Ok(SpinExpr::Up),
        "down" => Ok(SpinExpr::Down),
        _ => Err(ParseError(format!("Invalid spin: {}", s))),
    }
}
