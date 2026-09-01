use std::collections::BTreeMap;

pub fn sign(params: &BTreeMap<String, String>, secret: &str) -> String {
    let mut signature = String::new();

    for (key, value) in params {
        if key != "format" {
            signature.push_str(key);
            signature.push_str(value);
        }
    }

    let digest = md5::compute(format!("{}{}", signature, secret));
    format!("{:x}", digest)
}
