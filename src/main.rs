//! A bot that logs chat messages sent in the server to the console.

use std::ptr::hash;
use azalea::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use azalea::ecs::prelude::With;
use azalea::entity::Position;
use azalea::entity::metadata::Player;
use azalea::{BlockPos, GameProfileComponent};
use azalea::pathfinder::BlockPosGoal;

#[tokio::main]
async fn main() {
    let account = Account::offline("botbot");
    // or Account::microsoft("example@example.com").await.unwrap();

    loop {
        let e = ClientBuilder::new()
            .set_handler(handle)
            .start(account.clone(), "localhost")
            .await;
        eprintln!("{:?}", e);
    }
}

#[derive(Default, Clone, Component)]
pub struct State {}

async fn handle(mut bot: Client, event: Event, state: State) -> anyhow::Result<()> {
    match event {
        Event::Chat(m) => {
            let Some(sender) = m.username()  else { return Ok(()) };
                    let entity = bot.entity_by::<With<Player>, (&GameProfileComponent, )>(
                        |profile: &&GameProfileComponent| {
                            println!("entity {profile:?}");
                            profile.name == sender
                        },
                    );
                    if let Some(entity) = entity {
                        match m.content().as_str() {
                            "here" => {
                                let pos = bot.entity_component::<Position>(entity);
                                let target_pos: BlockPos = pos.into();
                                bot.goto(BlockPosGoal::from(target_pos));
                            }
                            _ => {}
                        }
                    }

        }
        Event::Login => {
            let a = &bot.profile.name;
            bot.chat(&*format!("/register {}ab", a));
            bot.chat(&*format!("/login {}ab", a));
            bot.chat("Hello world");
        }
        _ => {}
    }

    Ok(())
}