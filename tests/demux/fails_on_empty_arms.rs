use tokio::stream;

fn main() {
    let _ = async move {
        let stream = stream::iter::<()>(vec![]);
        demux! { stream -> }
    };
}
