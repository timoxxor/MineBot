//! A bot that logs chat messages sent in the server to the console.

use azalea::ecs::query::With;
use azalea::entity::metadata::Player;
use azalea::entity::{EyeHeight, Position};
use azalea::interact::HitResultComponent;
use azalea::inventory::ItemSlot;
use azalea::pathfinder::BlockPosGoal;
use azalea::{prelude::*, BlockPos, GameProfileComponent, WalkDirection, Vec3};
use azalea::{Account, Client, Event};
use std::time::Duration;
use std::env;
use tokio::sync::mpsc;
use std::io;
use std::io::Write;
use azalea::Item::String;
use azalea::protocol::packets::game::ClientboundGamePacket;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Please, provide 2 arguments");
        return;
    }
    let bot_name = &args[1];
    let ip = &args[2];

    let account = Account::offline(bot_name);
    // or Account::microsoft("example@example.com").await.unwrap();

    let rin = read_stdin();
    let state = State{
        rin
    };



    loop {
        let e = ClientBuilder::new()
            .set_handler(handle)
            .set_state(state.clone())
            .start(account.clone(), &**ip)
            .await;
        eprintln!("{:?}", e);
    }
}

#[derive(Default, Clone, Component)]
pub struct State {
    rin: mpsc::Receiver<String>
}

async fn handle(mut bot: Client, event: Event, mut state: State) -> anyhow::Result<()> {
    match event {
        Event::Init => {
            match state.rin.try_recv() {
                Ok(s) => {

                }
                _ => {}
            }
        }
        Event::Packet(p) => match p {
            ClientboundGamePacket::Disconnect(d) => {
                write_stdin("end")
            }
            _ => {}
        }
        Event::Login => {
            let a = &bot.profile.name;
            bot.chat(&*format!("/register {}ab", a));
            tokio::time::sleep(Duration::from_millis(50)).await;
            bot.chat(&*format!("/login {}ab", a));
            tokio::time::sleep(Duration::from_millis(50)).await;
            bot.chat("Hello world");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Event::Chat(m) => {
            println!("client chat message: {}", m.content());
            if m.content() == bot.profile.name {
                bot.chat("Bye");
                tokio::time::sleep(Duration::from_millis(50)).await;
                bot.disconnect();
            }
            let Some(sender) = m.username() else {
                return Ok(())
            };
            // let mut ecs = bot.ecs.lock();
            // let entity = bot
            //     .ecs
            //     .lock()
            //     .query::<&Player>()
            //     .iter(&mut ecs)
            //     .find(|e| e.name() == Some(sender));
            // let entity = bot.entity_by::<With<Player>>(|name: &Name| name == sender);
            let entity = bot.entity_by::<With<Player>, (&GameProfileComponent,)>(
                |profile: &&GameProfileComponent| {
                    println!("entity {profile:?}");
                    profile.name == sender
                },
            );
            println!("sender entity: {entity:?}");
            if let Some(entity) = entity {
                match m.content().as_str() {
                    "whereami" => {
                        let pos = bot.entity_component::<Position>(entity);
                        bot.chat(&format!("You're at {pos:?}",));
                    }
                    "whereareyou" => {
                        let pos = bot.position();
                        bot.chat(&format!("I'm at {pos:?}",));
                    }
                    "goto" => {
                        let entity_pos = bot.entity_component::<Position>(entity);
                        let target_pos: BlockPos = entity_pos.into();
                        println!("going to {target_pos:?}");
                        bot.goto(BlockPosGoal::from(target_pos));
                    }
                    "look" => {
                        let entity_pos = bot
                            .entity_component::<Position>(entity)
                            .up(bot.entity_component::<EyeHeight>(entity).into());
                        println!("entity_pos: {entity_pos:?}");
                        bot.look_at(entity_pos);
                    }
                    "jump" => {
                        bot.set_jumping(true);
                    }
                    "walk" => {
                        bot.walk(WalkDirection::Forward);
                    }
                    "stop" => {
                        bot.set_jumping(false);
                        bot.walk(WalkDirection::None);
                    }
                    "lag" => {
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                    "sethome" => {

                    }
                    "inventory" => {
                        println!("inventory: {:?}", bot.menu());
                    }
                    "findblock" => {
                        let target_pos = bot
                            .world()
                            .read()
                            .find_block(bot.position(), &azalea::Block::DiamondBlock.into());
                        bot.chat(&format!("target_pos: {target_pos:?}",));
                    }
                    "gotoblock" => {
                        let target_pos = bot
                            .world()
                            .read()
                            .find_block(bot.position(), &azalea::Block::DiamondBlock.into());
                        if let Some(target_pos) = target_pos {
                            // +1 to stand on top of the block
                            bot.goto(BlockPosGoal::from(target_pos.up(1)));
                        } else {
                            bot.chat("no diamond block found");
                        }
                    }
                    "lever" => {
                        let target_pos = bot
                            .world()
                            .read()
                            .find_block(bot.position(), &azalea::Block::Lever.into());
                        let Some(target_pos) = target_pos else {
                            bot.chat("no lever found");
                            return Ok(())
                        };
                        bot.goto(BlockPosGoal::from(target_pos));
                        bot.look_at(target_pos.center());
                        bot.block_interact(target_pos);
                    }
                    "hitresult" => {
                        let hit_result = bot.get_component::<HitResultComponent>();
                        bot.chat(&format!("hit_result: {hit_result:?}",));
                    }
                    "chest" => {
                        let target_pos = bot
                            .world()
                            .read()
                            .find_block(bot.position(), &azalea::Block::Chest.into());
                        let Some(target_pos) = target_pos else {
                            bot.chat("no chest found");
                            return Ok(())
                        };
                        bot.look_at(target_pos.center());
                        let container = bot.open_container(target_pos).await;
                        println!("container: {:?}", container);
                        if let Some(container) = container {
                            if let Some(contents) = container.contents() {
                                for item in contents {
                                    if let ItemSlot::Present(item) = item {
                                        println!("item: {:?}", item);
                                    }
                                }
                            } else {
                                println!("container was immediately closed");
                            }
                        } else {
                            println!("no container found");
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn sethome(vec: Vec3){

}

fn read_stdin() -> mpsc::Receiver<String> {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut res = String::new();
        loop {
            match io::stdin().read_line(&mut res) {
                Ok(s) => {
                    tx.send(res.clone())
                }
                Err(e) => {}
            }
        }
    });

    return rx
}

fn write_stdin(s: String) {
    let res = format!("{}\n",s);
    //io::stdout().write_all(String::format())
}

fn write_json_stdin(s: String, json: String) {
    let res = format!("{} {}\n",s,json);
    //io::stdout().write_all(String::format())
}