#![feature(register_tool)]
#![register_tool(hermittool)]

use hermit::*;

#[agent(secret)]
#[ensure(forall a, !a.K(pwd))]
pub fn register(username: String, pwd: String) {
    db::store(username, pwd)
}

#[agent(secret)]
#[forgets(unhashed)]
fn hash(unhashed: String) -> String {
    let digest = md5::compute(unhashed);
    format!("{:x}", digest)
    // unhashed
}

mod db;
