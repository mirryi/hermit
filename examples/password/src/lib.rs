#![feature(register_tool)]
#![register_tool(hermittool)]

use hermit::*;

#[agent(secret)]
#[ensure(agents a: !K[a: pwd])]
pub fn register(username: String, pwd: String) {
    let pwd = hash(pwd);
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
