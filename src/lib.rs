#[macro_use]
extern crate deno_core;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
extern crate bson;
extern crate redis;
extern crate serde;

use deno_core::CoreOp;
use deno_core::PluginInitContext;
use deno_core::{Buf, ZeroCopyBuf};
use futures::FutureExt;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

use std::{collections::HashMap, sync::Mutex, sync::MutexGuard};

mod command;

// mod util

lazy_static! {
    static ref CLIENTS: Mutex<HashMap<usize, Client>> = Mutex::new(HashMap::new());
    static ref CLIENT_ID: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Serialize)]
pub struct AsyncResult<T>
where
    T: Serialize,
{
    command: usize,
    data: T,
}

#[derive(Serialize, Deserialize)]
pub enum CommandType {
    ConnectWithOptions,
}

#[derive(Deserialize)]
pub struct CommandArgs {
    command_type: CommandType,
    command_id: Option<usize>,
    client_id: Option<usize>,
}

impl CommandArgs {
    fn new(data: &[u8]) -> CommandArgs {
        serde_json::from_slice(data).unwrap()
    }
}

pub struct Command {
    identity: CommandArgs,
    data: Option<ZeroCopyBuf>,
}

impl Command {
    fn new(identity: CommandArgs, data: Option<ZeroCopyBuf>) -> Command {
        Command { identity, data }
    }
    fn get_client(&self) -> Client {
        get_client(self.identity.client_id.unwrap())
    }
}

fn get_client(client_id: usize) -> Client {
    let map: MutexGuard<HashMap<usize, Client>> = CLIENTS.lock().unwrap();
    map.get(&client_id).unwrap().clone()
}

init_fn!(init);

fn init(context: &mut dyn PluginInitContext) {
    context.register_op("mongo_command", Box::new(op_command));
}

fn op_command(data: &[u8], zero_copy: Option<ZeroCopyBuf>) -> CoreOp {
    let args = CommandArgs::new(data);
    let executor = match args.command_type {
        CommandType::ConnectWithOptions => command::get_connection,
    };

    executor(Command::new(args, zero_copy))
}
