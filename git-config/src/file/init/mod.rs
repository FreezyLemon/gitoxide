use crate::file::{init, section, Metadata};
use crate::{parse, File};
use git_features::threading::OwnShared;

mod types;
pub use types::{Error, Options};

///
pub mod from_env;
///
pub mod from_paths;
///
pub mod includes;

impl<'a> File<'a> {
    /// Return an empty `File` with the given `meta`-data to be attached to all new sections.
    pub fn new(meta: impl Into<OwnShared<Metadata>>) -> Self {
        Self {
            frontmatter_events: Default::default(),
            frontmatter_post_section: Default::default(),
            section_lookup_tree: Default::default(),
            sections: Default::default(),
            section_id_counter: 0,
            section_order: Default::default(),
            meta: meta.into(),
        }
    }

    /// Instantiate a new `File` from given `input`, associating each section and their values with
    /// `meta`-data, while respecting `options`.
    pub fn from_bytes_no_includes(
        input: &'a [u8],
        meta: impl Into<OwnShared<Metadata>>,
        options: init::Options<'_>,
    ) -> Result<Self, init::Error> {
        let meta = meta.into();
        Ok(Self::from_parse_events_no_includes(
            parse::Events::from_bytes(input, options.to_event_filter())?,
            meta.clone(),
        ))
    }

    /// Instantiate a new `File` from given `events`, associating each section and their values with
    /// `meta`-data.
    pub fn from_parse_events_no_includes(
        parse::Events { frontmatter, sections }: parse::Events<'a>,
        meta: impl Into<OwnShared<Metadata>>,
    ) -> Self {
        let meta = meta.into();
        let mut this = File::new(OwnShared::clone(&meta));

        this.frontmatter_events = frontmatter;

        for section in sections {
            this.push_section_internal(crate::file::Section {
                header: section.header,
                body: section::Body(section.events),
                meta: OwnShared::clone(&meta),
            });
        }

        this
    }
}
