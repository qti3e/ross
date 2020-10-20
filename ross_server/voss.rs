pub mod server {
    use super::voss_runtime;
    use super::voss_runtime::{VossBuilder, VossReader};
    use super::{rpc, vcs, objects};
    use actix::prelude::Message;
    use actix::*;
    use actix_web::web::Bytes;
    use actix_web_actors::ws;
    use rand::{self, rngs::ThreadRng, Rng};
    use std::collections::{HashMap, HashSet};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    use rocksdb::DB;

    const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
    const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

    // Messages sent from websocket to editor;
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct EditorMessage(pub u32, pub rpc::RPCMessage);

    // ****************************** Editor ******************************
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct BinaryMessage(pub Bytes);

    #[derive(Message)]
    #[rtype(u32)]
    pub struct Connect {
        pub addr: Recipient<BinaryMessage>,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Disconnect {
        pub id: u32,
    }

    pub struct EditorServer {
        db: DB,
        sessions: HashMap<u32, Recipient<BinaryMessage>>,
        rng: ThreadRng,
        pub objects: HashMap<voss_runtime::HASH16, objects::RooObject>,
        pub live: Vec<vcs::VossAction>
    }

    impl EditorServer {
        pub fn open(path: &str) -> Result<EditorServer, ()> {
            Ok(EditorServer {
                db: DB::open_default(path).map_err(|_| { () })?,
                sessions: HashMap::new(),
                rng: rand::thread_rng(),
                objects: HashMap::new(),
                live: Vec::new()
            })
        }

        pub fn broadcast(
            &self,
            message: rpc::RPCMessage,
            skip_id: u32,
        ) {
            let msg = VossBuilder::serialize_enum(&message).unwrap();
            let data = Bytes::from(msg);
            for (id, addr) in &self.sessions {
                if *id != skip_id {
                    addr.do_send(BinaryMessage(data.clone()));
                }
            }
        }
    }

    impl Actor for EditorServer {
        type Context = Context<Self>;
    }

    impl Handler<Connect> for EditorServer {
        type Result = u32;

        fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
            let id = loop {
                let id = self.rng.gen::<u32>();
                if id > 0 && !self.sessions.contains_key(&id) {
                    break id;
                }
            };
            self.sessions.insert(id, msg.addr);
            id
        }
    }

    impl Handler<Disconnect> for EditorServer {
        type Result = ();

        fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
            self.sessions.remove(&msg.id);
        }
    }

    // ****************************** Session ******************************
    pub struct WsSession {
        id: u32,
        hb: Instant,
        editor: Addr<EditorServer>,
    }

    impl Actor for WsSession {
        type Context = ws::WebsocketContext<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            // Start sending heartbeat messages.
            self.hb(ctx);

            let addr = ctx.address();
            self.editor
                .send(Connect {
                    addr: addr.recipient(),
                })
                .into_actor(self)
                .then(|res, act, ctx| {
                    match res {
                        Ok(res) => {
                            act.id = res;
                            let msg =
                                rpc::RPCMessage::HostID(rpc::HostIDMessage { value: res });
                            ctx.binary(VossBuilder::serialize_enum(&msg).unwrap());
                        }
                        // something is wrong with the editor.
                        _ => ctx.stop(),
                    }
                    fut::ready(())
                })
                .wait(ctx);
        }

        fn stopping(&mut self, _: &mut Self::Context) -> Running {
            self.editor.do_send(Disconnect { id: self.id });
            Running::Stop
        }
    }

    impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
        fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
            let msg = match msg {
                Err(_) => {
                    ctx.close(None);
                    return;
                }
                Ok(msg) => msg,
            };

            let msg = match msg {
                ws::Message::Ping(msg) => {
                    self.hb = Instant::now();
                    ctx.pong(&msg);
                    return;
                }
                ws::Message::Pong(_) => {
                    self.hb = Instant::now();
                    return;
                }
                ws::Message::Close(_) | ws::Message::Continuation(_) | ws::Message::Text(_) => {
                    ctx.close(None);
                    return;
                }
                ws::Message::Binary(bin) => {
                    match VossReader::deserialize_enum::<rpc::RPCMessage>(&bin) {
                        Ok(msg) => msg,
                        Err(_) => {
                            ctx.close(None);
                            return;
                        }
                    }
                }
                ws::Message::Nop => return,
            };

            match msg {
                rpc::RPCMessage::Clock(_) => {
                    let result = rpc::RPCMessage::Clock(rpc::ClockMessage { timestamp: now() });
                    ctx.binary(VossBuilder::serialize_enum(&result).unwrap());
                }
                msg => {
                    self.editor.do_send(EditorMessage(self.id, msg));
                }
            }
        }
    }

    impl Handler<BinaryMessage> for WsSession {
        type Result = ();

        fn handle(&mut self, msg: BinaryMessage, ctx: &mut Self::Context) {
            ctx.binary(msg.0);
        }
    }

    impl WsSession {
        pub fn new(editor: Addr<EditorServer>) -> WsSession {
            WsSession {
                id: 0,
                hb: Instant::now(),
                editor,
            }
        }

        fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
            ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
                if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                    ctx.stop();
                    return;
                }
                ctx.ping(b"");
            });
        }
    }

    fn now() -> f64 {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        duration.as_millis() as f64
    }

    // Handle RPC messages sent from users to the editor server.
    impl Handler<EditorMessage> for EditorServer {
        type Result = ();

        #[inline(always)]
        fn handle(&mut self, EditorMessage(user, msg): EditorMessage, _: &mut Context<Self>) -> Self::Result {
            println!("{:?}", msg);
        }
    }
}