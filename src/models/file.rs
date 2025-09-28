pub struct File;

impl File {
    pub fn video_preview(subpath: &str, video: &str) -> String {
        let thumbnail_path = if !subpath.is_empty() {
            format!("/{}/thumbs/{}.webp", subpath, video)
        } else {
            format!("/thumbs/{}.webp", video)
        };
        format!(
            "<a href='/videos/{link}' style='max-width:250px;display:inline-block;' class='video_preview'>\
            <img src='{thumbnail_path}' class='img-fluid rounded border' alt='{video}' style='width:100%;'>\
            <div class='text-center text-white position-absolute mx-auto px-2 filename'>{filename}</div></a>",
            link = if subpath.is_empty() { format!("/{}", video) } else { format!("/{}/{}", subpath, video) },
            thumbnail_path = thumbnail_path,
            video = video,
            filename = video
        )
    }

    pub fn file_preview(link: &str, file_name: &str, is_video: bool) -> String {
        let main_link = if is_video {
            format!("/videos{}", link)
        } else {
            link.to_string()
        };
        let extra_link = if is_video {
            format!(" <a href='{}'>🎬</a>", link)
        } else {
            "".to_string()
        };
        format!(
            "<li class='list-group-item'><a href='{}'>{}</a>{}</li>",
            main_link, file_name, extra_link
        )
    }

    pub fn clip_preview(clip: &crate::models::clip::Model) -> String {
        format!(
            "<li class='list-group-item'>
            <a href='{source_filename}'>
            {source_filename}
            </a> &gt;
            <a href='/segments/{clip_filename}'>
            {clip_filename} ({start}-{end})
            </a>
            </li>",
            source_filename = clip.source_filename.clone(),
            clip_filename = clip.clip_filename,
            start = clip.start,
            end = clip.end,
        )
    }

    pub fn point_preview(point: &crate::models::point::Model) -> String {
        // Format time as HH:MM:SS:ms
        let total_ms = point.time;
        let ms = (total_ms % 1000) / 10; // two digits
        let total_seconds = total_ms / 1000;
        let s = total_seconds % 60;
        let total_minutes = total_seconds / 60;
        let m = total_minutes % 60;
        let h = total_minutes / 60;
        let formatted_time = format!("{:02}:{:02}:{:02}:{:02}", h, m, s, ms);
        let name = point.name.clone().unwrap_or_else(|| "Untitled".to_string());
        format!(
            "<li class='list-group-item'>
            <a href='{source_filename}#t={time_seconds}'>
            {source_filename}
            </a> &gt; Point {id}: {name} ({formatted_time})
            </li>",
            source_filename = point.source_filename.clone(),
            time_seconds = point.time as f64 / 1000.0,
            id = point.id,
            name = name,
            formatted_time = formatted_time,
        )
    }

    pub fn tag_preview(tag: &crate::models::tag::Model) -> String {
        format!(
            "<li class='list-group-item'>
            <a href='{source_filename}'>
            {source_filename}
            </a> &gt; Tag {id}: {tag}
            </li>",
            source_filename = tag.source_filename.clone(),
            id = tag.id,
            tag = tag.tag.clone()
        )
    }

    pub fn clip_video_preview(clip: &crate::models::clip::Model) -> String {
        let dir = std::path::Path::new(&clip.source_filename)
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let segments_path = if !dir.is_empty() {
            format!("/{}/segments/{}", dir, clip.clip_filename)
        } else {
            format!("/segments/{}", clip.clip_filename)
        };
        let thumb_path = if !dir.is_empty() {
            format!("/{}/segments/thumbs/{}.webp", dir, clip.clip_filename)
        } else {
            format!("/segments/thumbs/{}.webp", clip.clip_filename)
        };
        format!(
            "<a href='{segments_path}' style='max-width:250px;display:inline-block;' class='video_preview'>\
            <img src='{thumb_path}' class='img-fluid rounded border' alt='{clip_filename}' style='width:100%;'>\
            <div class='text-center text-white position-absolute mx-auto px-2 filename'>{clip_filename} ({start}-{end})</div></a>",
            segments_path = segments_path,
            thumb_path = thumb_path,
            clip_filename = clip.clip_filename,
            start = clip.start,
            end = clip.end,
        )
    }

    
}