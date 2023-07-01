//! Generate html pages for browsing the AskHistorians Archive.

// clippy WARN level lints
#![warn(
    clippy::cargo,
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::map_err_ignore,
    clippy::missing_docs_in_private_items,
    clippy::panic,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable
)]
// clippy WARN level lints, that can be upgraded to DENY if preferred
#![warn(
    clippy::float_arithmetic,
    clippy::integer_arithmetic,
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
    clippy::pattern_type_mismatch,
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
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_to_string,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads
)]

mod config;

use std::{collections::HashMap, io::BufRead};

pub use config::CONFIG;

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub fn run() -> Result<()> {
    let mut posts = read_posts()?;

    let mut comments = read_comments(&mut posts)?;

    for mut post in posts.values_mut() {
        if let Some(comment_list) = comments.remove(&post.id) {
            post.comments = Some(comment_list)
        } else {
            post.comments = Some(Vec::new())
        }
    }

    std::fs::create_dir_all("output/posts")?;

    let parser = make_template_parser()?;

    for post in posts.values() {
        render_post(&parser, post)?;
    }

    render_index(&parser, posts)?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    title: String,
    #[serde(rename(deserialize = "name"))]
    id: PostId,
    selftext: String,
    num_comments: u64,
    selftext_html: Option<String>,
    comments: Option<Vec<Comment>>,
}

type PostId = String;
type Posts = HashMap<PostId, Post>;

fn read_posts() -> Result<Posts> {
    let lines = std::io::BufReader::new(
        std::fs::File::open("submissions.json").context("Could not read submissions.json")?,
    )
    .lines()
    .take(500)
    .collect::<Vec<_>>();

    lines
        .into_par_iter()
        .map(|maybe_line| {
            maybe_line
                .context("could not read line")
                .and_then(|s| serde_json::from_str(&s).context("could not deserialize"))
        })
        .filter(|maybe_post| {
            maybe_post
                .as_ref()
                .map(|post: &Post| post.num_comments > 0)
                .unwrap_or(true)
        })
        .map(process_post)
        .collect::<Result<_, _>>()
}

fn process_post(post: Result<Post>) -> Result<(PostId, Post)> {
    let post = post?;

    let selftext_html = post
        .selftext_html
        .as_ref()
        .map(html_escape::decode_html_entities)
        .map(String::from);

    Ok((
        post.id.clone(),
        Post {
            selftext_html,
            ..post
        },
    ))
}

#[derive(Serialize, Deserialize, Debug)]
struct Comment {
    link_id: String,
    parent_id: String,
    body: String,
}

type Comments = HashMap<PostId, Vec<Comment>>;

fn read_comments(posts: &mut Posts) -> Result<Comments> {
    let lines = std::io::BufReader::new(
        std::fs::File::open("comments.json").context("Could not read comments.json")?,
    )
    .lines()
    .take(50_000)
    .collect::<Vec<_>>();

    let comments = lines
        .into_par_iter()
        .map(
            |maybe_line: Result<String, std::io::Error>| -> Result<Comment> {
                let line = maybe_line.context("could not read line")?;
                let comment: Comment =
                    serde_json::from_str(&line).context("could not deserialize comment")?;

                Ok(comment)
            },
        )
        .filter(|comment| {
            comment
                .as_ref()
                .map(|c| posts.contains_key(&c.parent_id))
                .unwrap_or(true)
        })
        .try_fold(
            HashMap::new,
            |mut comments: Comments, comment| -> Result<Comments> {
                let comment = comment?;
                comments
                    .entry(comment.link_id.clone())
                    .or_insert(Vec::new())
                    .push(comment);

                Ok(comments)
            },
        )
        .try_reduce(HashMap::new, |mut comments, other_comments| {
            for (link_id, mut new_comment_list) in other_comments {
                comments
                    .entry(link_id)
                    .or_insert(Vec::new())
                    .append(&mut new_comment_list)
            }
            Ok(comments)
        })?;

    Ok(comments)
}

fn make_template_parser() -> Result<liquid::Parser> {
    let mut partial_source = liquid::partials::InMemorySource::new();

    partial_source.add("head", std::fs::read_to_string("templates/head.liquid")?);
    partial_source.add("nav", std::fs::read_to_string("templates/nav.liquid")?);
    partial_source.add(
        "layout",
        std::fs::read_to_string("templates/layout.liquid")?,
    );

    let partials = liquid::partials::EagerCompiler::new(partial_source);

    liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .context("Could not build liquid parser")
}

fn render_index(parser: &liquid::Parser, posts: Posts) -> Result<()> {
    let template = parser.parse_file("templates/index.liquid")?;

    let globals =
        liquid::object!({ "posts": liquid::model::to_value(&posts.values().collect::<Vec<_>>())? });

    let output = template.render(&globals)?;

    std::fs::write("output/index.html", output)?;

    Ok(())
}

fn render_post(parser: &liquid::Parser, post: &Post) -> Result<()> {
    let template = parser.parse_file("templates/post.liquid")?;

    let globals = liquid::to_object(&post)?;

    let output = template.render(&globals)?;
    let name = &post.id;
    std::fs::write(format!("output/posts/{name}.html"), output)?;

    Ok(())
}
