use zbus::{ConnectionBuilder, Result};

mod disk;
mod zbus_server;

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
pub async fn main() -> Result<()> {
    let server_manager = zbus_server::ServerManager {
        done: event_listener::Event::new(),
    };
    let done_listener = server_manager.done.listen();
    let _ = ConnectionBuilder::system()?
        .name("com.subgraph.installer")?
        .serve_at("/com/subgraph/installer", server_manager)?
        .build()
        .await?;

    done_listener.wait();

    //pending::<()>().await;
    Ok(())
}
