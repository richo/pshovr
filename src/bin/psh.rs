use pshovr;
use std::io;
use std::env;
use std::error::Error;

/// Retrieve the payload for this notification.
///
/// In order, it will attempt:
///  - Command line arguments given to the binary
///  - Reading from stdin
fn get_payload() -> Option<String> {
    let args: Vec<_> = env::args().collect();
    let mut stdin = io::stdin().lock();
    return _get_payload(&args, &mut stdin);
}
fn _get_payload(args: &Vec<String>, stdin: &mut dyn io::BufRead) -> Option<String> {
    // TODO(richo) inject args and stdin so we can test this
    let cli_args = &args[1..];
    return match cli_args.len() {
        0 => _get_stdin_payload(stdin),
        1 => Some(cli_args[0].clone()),
        _ => Some(cli_args.join(" ")),
    };
}

fn _get_stdin_payload(stdin: &mut dyn io::BufRead) -> Option<String> {
    // TODO(richo) this really might not be how we wanna tackle this
    let mut buf = String::new();

    stdin.read_line(&mut buf).ok()?;

    if buf.len() == 0 {
        return None;
    }
    return Some(buf);
}

fn main() -> Result<(), Box<dyn Error>> {
    let pushover = pshovr::PushoverClient::new(
        env::var("PUSHOVER_API_KEY")
            .map_err(|e| format!("Didn't get api key from PUSHOVER_API_KEY: {:?}", e))?
    );
    let user_key: String = env::var("PUSHOVER_USER_KEY")
            .map_err(|e| format!("Didn't get user key from PUSHOVER_USER_KEY: {:?}", e))?;
    let payload = get_payload()
        .ok_or("Couldn't get payload.".to_string())?;
    let req = pushover.build_notification(&user_key, payload.trim_end());
    pushover.send(&req)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prioritizes_args() {
        let args = vec!("psh".into(), "hello yes".into());
        let mut stdin = "ignored".to_string();

        let payload = _get_payload(&args, &mut stdin.as_bytes());
        assert_eq!(payload, Some("hello yes".into()));
    }

    #[test]
    fn test_uses_stdin() {
        let args = vec!("psh".into());
        let mut stdin = "this is the input".to_string();

        let payload = _get_payload(&args, &mut stdin.as_bytes());
        assert_eq!(payload, Some("this is the input".into()));
    }

    #[test]
    fn test_flattens_args() {
        let args = vec!("psh".into(), "hello".into(), "yes".into(), "!!!".into());
        let mut stdin = "ignored".to_string();

        let payload = _get_payload(&args, &mut stdin.as_bytes());
        assert_eq!(payload, Some("hello yes !!!".into()));
    }

    #[test]
    fn test_returns_none_with_no_input() {
        let args = vec!("psh".into());
        let mut stdin = "".to_string();

        let payload = _get_payload(&args, &mut stdin.as_bytes());
        assert_eq!(payload, None);
    }
}
