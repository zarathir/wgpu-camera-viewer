use std::thread;

use anyhow::Result;

use webcam_viewer::UserEvent;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;
use zenoh::prelude::r#async::AsyncResolve;
use zenoh::prelude::Encoding;
use zenoh::prelude::KnownEncoding;
use zenoh::prelude::SplitBuffer;

fn main() -> Result<()> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1600, 900))
        .with_resizable(true)
        .with_title("Webcam viewer")
        .build(&event_loop)?;

    thread::spawn(move || {
        async_std::task::block_on(async move {
            let config = zenoh::config::peer();
            let session = zenoh::open(config).res().await.unwrap();

            let subscriber = session
                .declare_subscriber("camera_node")
                .res()
                .await
                .unwrap();

            loop {
                let buf = subscriber.recv_async().await.unwrap();
                let encoding = buf
                    .value
                    .encoding(Encoding::Exact(KnownEncoding::AppOctetStream));
                let image = encoding.payload.contiguous();
                proxy
                    .send_event(UserEvent::NewImage(image.to_vec()))
                    .unwrap();
            }
        });
    });

    pollster::block_on(webcam_viewer::run(event_loop, window));
    Ok(())
}
