// use hermit_attributes::*;

// #[agent(secret)]
// #[ensure(forall a, !a.K(pwd))]
pub fn register(username: String, pwd: String) {
    store(username, pwd)
}

// #[agent(secret)]
// #[forgets(unhashed)]
fn hash(unhashed: String) -> String {
    // let digest = md5::compute(unhashed);
    // format!("{:x}", digest)
    unhashed
}

fn store(_username: String, _pwd_hash: String) {
    // println!("store {}: {}", username, pwd_hash)
}
