#[hermit::agent(secret)]
#[hermit::ensure(forall a, !a.K(pwd))]
pub fn register(username: String, pwd: String) {
    db::store(username, pwd)
}

#[hermit::agent(secret)]
#[hermit::forgets(unhashed)]
fn hash(unhashed: String) -> String {
    let digest = md5::compute(unhashed);
    format!("{:x}", digest)
    // unhashed
}

mod db;
