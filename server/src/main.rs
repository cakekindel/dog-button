use kwap::{blocking::server::Server, platform::Std};

mod service {
    use kwap::{
        blocking::server::{
            Action::{Continue, SendReq, SendResp},
            Actions,
        },
        net::Addrd,
        platform::Std,
        req::{Method, Req},
        resp::{code, Resp},
    };
    use rodio::OutputStream;
    use std::{
        fs::File,
        io::BufReader,
        time::{Duration, Instant}, thread::sleep,
    };

    static mut LAST_BROADCAST: Option<Instant> = None;

    pub fn post_pressed(req: &Addrd<Req<Std>>) -> Actions<Std> {
        match (
            req.data().method(),
            req.data().path().unwrap().unwrap_or_default(),
        ) {
            (Method::GET, "pressed") => {
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();

                let file = BufReader::new(File::open("screm.wav").unwrap());
                let sink = stream_handle.play_once(file).unwrap();
                sink.set_volume(0.15);
                sleep(Duration::from_millis(1000));

                let resp =
                    req.as_ref()
                        .map(Resp::for_request)
                        .map(Option::unwrap)
                        .map(|mut rep| {
                            rep.set_code(code::CONTENT);
                            rep.set_payload("ok".bytes());
                            rep
                        });

                SendResp(resp)
            }
            _ => Continue,
        }
        .into()
    }

    pub fn send_multicast_broadcast() -> Actions<Std> {
        match unsafe { LAST_BROADCAST } {
            Some(inst) if inst > (Instant::now() - Duration::from_millis(1000)) => {
                Actions::just(Continue)
            }
            _ => {
                let addr = kwap::multicast::all_coap_devices(1234);

                let mut req = Req::<Std>::post(addr, "");
                req.non();

                unsafe {
                    LAST_BROADCAST = Some(Instant::now());
                }

                SendReq(Addrd(req, addr)).then(Continue)
            }
        }
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Trace).unwrap();
    let mut server = kwap::blocking::Server::try_new([0, 0, 0, 0], 1111).unwrap();

    server.middleware(&service::post_pressed);

    server.start_tick(Some(&service::send_multicast_broadcast));
}
