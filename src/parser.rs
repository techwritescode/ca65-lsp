pub fn parse_ident(ident: &str) -> String {
    if ident.ends_with(":") {
        return ident[0..ident.len() - 1].to_string();
    }

    ident.to_string()
}

pub fn is_identifier(ident: &str) -> bool {
    for (i, char) in ident.as_bytes().iter().enumerate() {
        if !(char.is_ascii_alphanumeric() || *char == b'_') {
            if (*char == b':' && i == ident.len() - 1) || (i == 0 && *char == b'@') {
            } else {
                // tracing::error!("Failed {}", ident);
                return false;
            }
        }
    }
    tracing::error!("{}", ident);
    true
}
