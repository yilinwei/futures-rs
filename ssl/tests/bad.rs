extern crate futures;
extern crate env_logger;
extern crate futuremio;
extern crate ssl;

#[macro_use]
extern crate cfg_if;

use std::io::Error;
use std::net::ToSocketAddrs;

use futures::Future;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

cfg_if! {
    if #[cfg(any(feature = "force-openssl",
                 all(not(target_os = "macos"),
                     not(target_os = "windows"))))] {
        extern crate openssl;

        use openssl::ssl as ossl;

        fn get(err: &Error) -> &[ossl::error::OpenSslError] {
            let err = err.get_ref().unwrap();
            match *err.downcast_ref::<ossl::error::Error>().unwrap() {
                ossl::Error::Ssl(ref v) => v,
                ref e => panic!("not an ssl eror: {:?}", e),
            }
        }

        fn verify_failed(err: &Error) {
            assert!(get(err).iter().any(|e| {
                e.reason() == "certificate verify failed"
            }), "bad errors: {:?}", err);
        }

        use verify_failed as assert_expired_error;
        use verify_failed as assert_wrong_host;
        use verify_failed as assert_self_signed;
        use verify_failed as assert_untrusted_root;

        fn assert_dh_too_small(err: &Error) {
            assert!(get(err).iter().any(|e| {
                e.reason() == "dh key too small"
            }), "bad errors: {:?}", err);
        }
    } else if #[cfg(target_os = "macos")] {
        extern crate security_framework;

        use security_framework::base::Error as SfError;

        fn assert_expired_error(err: &Error) {
            let err = err.get_ref().unwrap();
            let err = err.downcast_ref::<SfError>().unwrap();
            assert_eq!(err.message().unwrap(), "invalid certificate chain");
        }

        fn assert_wrong_host(err: &Error) {
            let err = err.get_ref().unwrap();
            let err = err.downcast_ref::<SfError>().unwrap();
            assert_eq!(err.message().unwrap(), "invalid certificate chain");
        }

        fn assert_self_signed(err: &Error) {
            let err = err.get_ref().unwrap();
            let err = err.downcast_ref::<SfError>().unwrap();
            assert_eq!(err.message().unwrap(), "invalid certificate chain");
        }

        fn assert_untrusted_root(err: &Error) {
            let err = err.get_ref().unwrap();
            let err = err.downcast_ref::<SfError>().unwrap();
            assert_eq!(err.message().unwrap(), "invalid certificate chain");
        }

        fn assert_dh_too_small(err: &Error) {
            let err = err.get_ref().unwrap();
            let err = err.downcast_ref::<SfError>().unwrap();
            assert_eq!(err.message().unwrap(), "invalid certificate chain");
        }
    } else {
        extern crate winapi;

        fn assert_expired_error(err: &Error) {
            let code = err.raw_os_error().unwrap();
            assert_eq!(code as usize, winapi::CERT_E_CN_NO_MATCH as usize);
        }

        fn assert_wrong_host(err: &Error) {
            let code = err.raw_os_error().unwrap();
            assert_eq!(code as usize, winapi::CERT_E_CN_NO_MATCH as usize);
        }

        fn assert_self_signed(err: &Error) {
            let code = err.raw_os_error().unwrap();
            assert_eq!(code as usize, winapi::CERT_E_CN_NO_MATCH as usize);
        }

        fn assert_untrusted_root(err: &Error) {
            let code = err.raw_os_error().unwrap();
            assert_eq!(code as usize, winapi::CERT_E_CN_NO_MATCH as usize);
        }

        fn assert_dh_too_small(err: &Error) {
            let code = err.raw_os_error().unwrap();
            assert_eq!(code as usize, winapi::CERT_E_CN_NO_MATCH as usize);
        }
    }
}

fn get_host(host: &'static str) -> Error {
    drop(env_logger::init());

    let addr = format!("{}:443", host);
    let addr = t!(addr.to_socket_addrs()).next().unwrap();

    let mut l = t!(futuremio::Loop::new());
    let client = l.handle().tcp_connect(&addr);
    let data = client.and_then(move |socket| {
        t!(ssl::ClientContext::new()).handshake(host, socket)
    });

    let res = l.run(data);
    assert!(res.is_err());
    res.err().unwrap()
}

#[test]
fn expired() {
    assert_expired_error(&get_host("expired.badssl.com"))
}

#[test]
fn wrong_host() {
    assert_wrong_host(&get_host("wrong.host.badssl.com"))
}

#[test]
fn self_signed() {
    assert_self_signed(&get_host("self-signed.badssl.com"))
}

#[test]
fn untrusted_root() {
    assert_untrusted_root(&get_host("untrusted-root.badssl.com"))
}

#[test]
fn dh_too_small() {
    assert_dh_too_small(&get_host("dh480.badssl.com"))
}
