pub struct File;

impl File {
    pub fn video_preview(subpath: &str, video: &str) -> String {
        format!(
            "<a href='/videos/{link}' style='max-width:250px;display:inline-block;'>\
            <img src='{subpath}/thumbs/{video}.webp' class='img-fluid rounded border' alt='{video}' style='width:100%;'>\
            <div class='text-center text-white position-absolute mx-auto px-2' style='margin-top:-30px;'>{filename}</div></a>",
            link = if subpath.is_empty() { format!("/{}", video) } else { format!("/{}/{}", subpath, video) },
            subpath = subpath,
            video = video,
            filename = video
        )
    }

    pub fn file_preview(link: &str, file_name: &str, is_video: bool) -> String {
        if is_video {
            format!(
                "<li class='list-group-item'><a href='{}'>{}</a> <a href='/videos{}'>🎬</a></li>",
                link, file_name, link
            )
        } else {
            format!("<li class='list-group-item'><a href='{}'>{}</a></li>", link, file_name)
        }
    }
}