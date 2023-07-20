//! Generate HTML pages for browsing the Ask Historians Reddit Archive.

// clippy WARN level lints
#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::map_err_ignore,
    clippy::panic,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable
)]
// clippy WARN level lints, that can be upgraded to DENY if preferred
#![warn(
    clippy::modulo_arithmetic,
    clippy::as_conversions,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::string_slice,
    clippy::try_err
)]
// clippy DENY level lints, they always have a quick fix that should be preferred
#![deny(
    clippy::wildcard_imports,
    clippy::multiple_inherent_impl,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::separated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::string_add,
    clippy::string_to_string,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads
)]

pub mod config;
mod render;

use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use config::Config;
use html_escape::decode_html_entities;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[tracing::instrument]
pub fn run(config: Config) -> Result<()> {
    let default_limit = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
    let limit = config.limit_posts.map_or(default_limit, |limit| {
        Utc.from_local_datetime(
            &limit
                .and_hms_opt(0, 0, 0)
                .expect("Failed to convert date to datetime"),
        )
        .unwrap()
    });
    let mut posts = read_posts(&config.submissions, limit);
    info!("Found posts: {}", posts.len());

    read_comments(&config.comments, &mut posts, limit)?;

    std::fs::remove_dir_all("output")?;
    std::fs::create_dir_all("output/posts")?;
    std::fs::create_dir_all("output/pages")?;

    debug!("Cleaning up read data");
    let mut posts_to_render = posts
        .into_values()
        .par_bridge()
        .filter(|p| !p.comments.is_empty())
        .map(render::Post::from)
        .collect::<Vec<_>>();
    posts_to_render.sort_by_key(|post| post.created_at);
    posts_to_render.reverse();

    let total_posts: u32 = posts_to_render.len().try_into()?;
    debug!("Rendering {total_posts} posts");
    posts_to_render.par_iter().for_each(|post| {
        render::post(post).expect("Failed to render post");
    });

    debug!("Rendering listings");
    let page_size: u32 = 25;
    #[allow(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let total_pages = (f64::from(total_posts) / f64::from(page_size)).ceil() as usize + 1;
    for (index, page_posts) in posts_to_render.chunks(page_size.try_into()?).enumerate() {
        render::listing(page_posts, index + 1, total_pages)?;
    }

    render::index()?;
    render::about(limit)?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    title: String,
    author: String,
    id: PostId,
    num_comments: i64,
    selftext: String,
    selftext_html: Option<String>,
    #[serde(default)]
    comments: Vec<Comment>,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_utc: DateTime<Utc>,
    removed_by_category: Option<String>,
}

impl From<Post> for render::Post {
    fn from(mut post: Post) -> Self {
        let real_num_comments = post.comments.len();

        let selftext_html = post
            .selftext_html
            .map(|text| html_escape::decode_html_entities(&text).to_string());

        let selftext = html_escape::decode_html_entities(&post.selftext).to_string();

        let title = html_escape::decode_html_entities(&post.title).to_string();

        post.comments.sort_unstable_by_key(|c| -c.score);
        let comments = post
            .comments
            .into_iter()
            .map(render::Comment::from)
            .collect();

        Self {
            real_num_comments,
            selftext_html,
            selftext,
            id: post.id,
            title,
            author: post.author,
            comments,
            created_at: post.created_utc,
        }
    }
}

type PostId = String;
type Posts = HashMap<PostId, Post>;

#[tracing::instrument]
fn read_posts(path: &PathBuf, limit: DateTime<Utc>) -> Posts {
    let limit_description = limit.to_string();

    debug!("Reading posts after {limit_description} from {path:?}");

    let lines: Vec<_> =
        std::io::BufReader::new(std::fs::File::open(path).expect("Could not read {path:?}"))
            .lines()
            .collect();

    lines
        .into_par_iter()
        .map(|maybe_line| {
            let line = maybe_line.expect("could not read line");

            let post: Post = serde_json::from_str(&line)
                .context(line)
                .expect("could not deserialize");

            (post.id.clone(), post)
        })
        .filter(|(_id, post)| {
            post.num_comments > 0
                && post.created_utc > limit
                && post.removed_by_category.is_none()
                && post.selftext != "[removed]"
        })
        .collect()
}

#[derive(Serialize, Deserialize, Debug)]
struct Comment {
    parent_id: String,
    body: String,
    author: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_utc: DateTime<Utc>,
    score: i64,
}

impl From<Comment> for render::Comment {
    fn from(comment: Comment) -> Self {
        let body = decode_html_entities(&comment.body).to_string();
        Self {
            body,
            author: comment.author,
        }
    }
}

#[tracing::instrument(skip(posts))]
fn read_comments(path: &PathBuf, posts: &mut Posts, limit: DateTime<Utc>) -> Result<()> {
    let limit_description = limit.to_string();

    debug!("Reading comments after {limit_description} from {path:?}");

    let lines: Vec<_> =
        std::io::BufReader::new(std::fs::File::open(path).context("Could not read {path:?}")?)
            .lines()
            .collect();

    let posts_wrapper = Arc::new(Mutex::new(posts));

    lines
        .into_par_iter()
        .map(|maybe_line: Result<String, std::io::Error>| -> Comment {
            let line = maybe_line.expect("could not read line");

            let mut comment: Comment =
                serde_json::from_str(&line).expect("could not deserialize comment");
            comment.parent_id = comment.parent_id.replace("t3_", "");
            comment
        })
        .filter(|comment| {
            comment.body != "[deleted]"
                && comment.body != "[removed]"
                && comment.author != "AutoModerator"
                && comment.created_utc > limit
        })
        .for_each(|comment| {
            if let Some(post) = posts_wrapper
                .lock()
                .expect("Failed to get lock")
                .get_mut(&comment.parent_id)
            {
                post.comments.push(comment);
            }
        });

    Ok(())
}
