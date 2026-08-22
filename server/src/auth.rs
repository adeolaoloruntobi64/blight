use std::{collections::HashMap, io, path::Path};


pub fn parse_auth_file(auth_path: impl AsRef<Path>) -> Result<HashMap<String, String>, io::Error> { 
    let mut users = HashMap::new();
    let file = std::fs::read_to_string(auth_path)?;
    for entry in file.split('\n').map(str::trim).filter(|x| !x.is_empty()) {
        let Some((username, password)) = entry.split_once(":") else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("The auth '{entry}' does not contain a username and password component, skipping")
            ));
        };
        println!(
            "adding username {:?} password {:?} to allowed auth",
            username, password
        );
        users.insert(username.to_string(), password.to_string());
    }
    Ok(users)
}