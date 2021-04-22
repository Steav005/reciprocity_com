#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "host")]
pub mod host;

pub mod messages;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{PlayerState, BotInfo, Track, PlayMode};
    use serde::Serialize;
    use std::time::Duration;
    use serde_diff::{Diff, Apply};

    #[test]
    fn diff() {
        let old = PlayerState{
            bot: BotInfo {
                name: "BotName".to_string(),
                avatar: "Avatar".to_string()
            },
            paused: false,
            mode: PlayMode::Normal,
            current: None,
            history: vec![
                Track{
                    len: Duration::from_secs(5),
                    pos: Duration::from_secs(3),
                    title: "t1".to_string(),
                    uri: "u1".to_string()
                },
                Track{
                    len: Duration::from_secs(8),
                    pos: Duration::from_secs(2),
                    title: "t2".to_string(),
                    uri: "u1".to_string()
                }
            ],
            queue: vec![]
        };
        let new = PlayerState{
            bot: BotInfo {
                name: "BotName".to_string(),
                avatar: "Avatar".to_string()
            },
            paused: false,
            mode: PlayMode::Normal,
            current: None,
            history: vec![
                Track{
                    len: Duration::from_secs(5),
                    pos: Duration::from_secs(4),
                    title: "t1".to_string(),
                    uri: "u1".to_string()
                }
            ],
            queue: vec![]
        };
        let diff = rmp_serde::to_vec(&Diff::serializable(&old, &new)).unwrap();
        let mut patch = rmp_serde::Deserializer::new(diff.as_slice());
        let mut patched = old.clone();
        Apply::apply(&mut patch, &mut patched).unwrap();
        assert_eq!(patched, new);
    }
}
