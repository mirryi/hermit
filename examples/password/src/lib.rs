#![feature(register_tool)]
#![register_tool(hermittool)]

use hermit::*;

#[agent(secret)]
#[ensure(forall a: !K[a: pwd])]
pub fn register(username: String, pwd: String) {
    db::store(username, pwd)
}

#[agent(secret)]
#[forget(digest: unhashed)]
fn hash(unhashed: String) -> String {
    let digest = md5::compute(unhashed);
    format!("{:x}", digest)
    // unhashed
}

mod db;
