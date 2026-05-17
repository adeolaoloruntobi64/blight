use std::{collections::HashMap, io, path::Path};


pub fn parse_auth_file(auth_path: impl AsRef<Path>) -> Result<HashMap<String, String>, io::Error> { 
    let mut users = HashMap::new();
    let file = std::fs::read_to_string(auth_path)?;
    for entry in file.split('\n').filter_map(|x| {
        if x.contains(':') {
            Some(x.trim())
        } else {
            None
        }
    }) {
        let split: Vec<_> = entry.split(':').collect();
        if split.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("The auth '{entry}' does not contain a username and password component, skipping")
            ));
        }
        let username = split[0].to_string();
        let password = split[1..].join(":");
        println!(
            "adding username {:?} password {:?} to allowed auth",
            username, password
        );
        users.insert(username, password);
    }
    Ok(users)
}