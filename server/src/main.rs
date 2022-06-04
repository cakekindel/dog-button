use kwap::blocking::server::Server;
use rodio::OutputStream;
use std::{
    fs::File,
    io::BufReader,
    thread::sleep,
    time::Duration,
};

fn scream() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("screm.wav").unwrap());
    let sink = stream_handle.play_once(file).unwrap();
    sink.set_volume(0.15);
    sleep(Duration::from_millis(1000));
}

mod service {
    use super::*;
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
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    mod broadcast {
      use super::*;

      static mut LAST_BROADCAST: Option<Instant> = None;

      pub(super) fn last() -> Option<Instant> {
        unsafe {LAST_BROADCAST}
      }

      pub(super) fn set_now() {
        unsafe {LAST_BROADCAST = Some(Instant::now());}
      }
    }

    pub fn post_pressed(req: &Addrd<Req<Std>>) -> Actions<Std> {
        match (
            req.data().method(),
            req.data().path().unwrap().unwrap_or_default(),
        ) {
            (Method::POST, "pressed") => {
                scream();

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
        match broadcast::last() {
            Some(inst) if inst > (Instant::now() - Duration::from_millis(1000)) => {
                sleep(Duration::from_millis(10));
                Actions::just(Continue)
            }
            _ => {
                let addr = kwap::multicast::all_coap_devices(1234);

                let mut req = Req::<Std>::post(addr, "");
                req.non();

                broadcast::set_now();

                SendReq(Addrd(req, addr)).then(Continue)
            }
        }
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Trace).unwrap();
    let mut server = Server::try_new([0, 0, 0, 0], 1111).unwrap();

    server.middleware(&service::post_pressed);

    server.start_tick(Some(&service::send_multicast_broadcast)).unwrap();
}
