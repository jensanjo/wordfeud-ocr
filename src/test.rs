#[macro_export]
macro_rules! four {
    () => {
        1 + 3
    };
}

#[macro_export]
macro_rules! template {
    ($x:expr) => {
        concat!("templates/", $x, ".png")
    };
}

#[macro_export]
macro_rules! templates {
    ( $( $x:expr ),* ) => {
            [$(
                   ($x, include_bytes!(concat!("templates/", $x, ".png"))),
            )*]
        };
}

#[allow(dead_code)]
const TEMPLATES: &[(&str, &[u8])] = &templates![
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z", "Æ", "Å", "Ä", "Ñ", "Ö", "Ø", "CH", "LL", "RR"
];
// pub const TEMPLATES: &[&[u8]] = &templates!["A", "B", "X"];

#[cfg(test)]
mod tests {
    use super::TEMPLATES;

    // use super::*;

    #[test]
    fn test_four() {
        let n = four!();
        println!("n: {}", n);
    }

    #[test]
    fn test_template() {
        let template = template!("A");
        println!("{}", template);
    }

    #[test]
    fn test_templates() {
        // let mut templates = Vec::new();
        // for (name, buf) in TEMPLATES.iter() {
        //     println!("{} {}", name, buf.len());

        // }
        let templates: Vec<_> = TEMPLATES
            .iter()
            .map(|(name, buf)| (name, image::load_from_memory(buf).unwrap().to_luma8()))
            .collect();
        let (&name, template) = &templates[0];
        println!("{} {:?}", name, template.dimensions());
        // let img = image::load_from_memory(TEMPLATES[0].1).unwrap();
        // let gray = img.to_luma8();
        // gray.save("A.png").unwrap();
    }
}
