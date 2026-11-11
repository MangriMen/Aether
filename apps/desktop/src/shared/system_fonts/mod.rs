use std::collections::BTreeSet;

use fontdb::{Database, FaceInfo, ID};
use tokio::sync::OnceCell;

/// The installed font families the interface can use for its text, by name.
///
/// Scanning the fonts takes hundreds of milliseconds, so it is done once per run: fonts
/// installed meanwhile show after a restart.
pub async fn list_system_font_families() -> Result<&'static [String], tauri::Error> {
    static FAMILIES: OnceCell<Vec<String>> = OnceCell::const_new();

    FAMILIES
        .get_or_try_init(|| async {
            tauri::async_runtime::spawn_blocking(scan_system_font_families).await
        })
        .await
        .map(Vec::as_slice)
}

fn scan_system_font_families() -> Vec<String> {
    let mut db = Database::new();
    db.load_system_fonts();

    collect_families(&db, |id| {
        db.with_face_data(id, face_has_latin).unwrap_or(false)
    })
}

/// A face that cannot show plain Latin text is a symbol font (Wingdings and the like): chosen
/// for the interface, it would make the text unreadable.
fn face_has_latin(data: &[u8], index: u32) -> bool {
    ttf_parser::Face::parse(data, index)
        .is_ok_and(|face| ['A', 'a'].iter().all(|&ch| face.glyph_index(ch).is_some()))
}

/// The names of the families of the database, without repeats, in alphabetical order. A family
/// is named by its first name (English, unless the font has none): the same family under its
/// localized name is not listed twice.
fn collect_families(db: &Database, has_latin: impl Fn(ID) -> bool) -> Vec<String> {
    let mut families = BTreeSet::new();

    for face in db.faces() {
        let Some(name) = primary_family(face) else {
            continue;
        };

        // A family is listed by its first face with Latin; the check opens the file, so the
        // families already listed are not asked again.
        if !families.contains(name) && has_latin(face.id) {
            families.insert(name.to_owned());
        }
    }

    families.into_iter().collect()
}

fn primary_family(face: &FaceInfo) -> Option<&str> {
    face.families
        .first()
        .map(|(name, _)| name.trim())
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use fontdb::{Language, Source, Stretch, Style, Weight};

    use super::*;

    fn push_face(db: &mut Database, families: &[&str]) -> ID {
        db.push_face_info(FaceInfo {
            id: ID::dummy(),
            source: Source::Binary(std::sync::Arc::new(Vec::<u8>::new())),
            index: 0,
            families: families
                .iter()
                .map(|name| ((*name).to_owned(), Language::English_UnitedStates))
                .collect(),
            post_script_name: String::new(),
            style: Style::Normal,
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            monospaced: false,
        })
    }

    #[test]
    fn lists_each_family_once_in_alphabetical_order() {
        let mut db = Database::new();
        push_face(&mut db, &["Zeta"]);
        push_face(&mut db, &["Alpha"]);
        push_face(&mut db, &["Alpha"]);

        assert_eq!(collect_families(&db, |_| true), ["Alpha", "Zeta"]);
    }

    #[test]
    fn names_a_family_by_its_first_name() {
        let mut db = Database::new();
        push_face(&mut db, &["Arial", "Арial локализованный"]);

        assert_eq!(collect_families(&db, |_| true), ["Arial"]);
    }

    #[test]
    fn skips_faces_without_latin_and_without_a_name() {
        let mut db = Database::new();
        let symbols = push_face(&mut db, &["Wingdings"]);
        push_face(&mut db, &["Fira Sans"]);
        push_face(&mut db, &["  "]);
        push_face(&mut db, &[]);

        assert_eq!(collect_families(&db, |id| id != symbols), ["Fira Sans"]);
    }

    #[test]
    fn lists_a_family_when_only_a_later_face_has_latin() {
        let mut db = Database::new();
        let first = push_face(&mut db, &["Mixed"]);
        push_face(&mut db, &["Mixed"]);

        assert_eq!(collect_families(&db, |id| id != first), ["Mixed"]);
    }

    #[test]
    fn symbols_have_no_latin_to_show() {
        assert!(!face_has_latin(b"not a font", 0));
    }
}
