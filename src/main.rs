extern crate clipboard;
extern crate regex;
extern crate inputbot;

use inputbot::*;
use KeybdKey::*;

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use regex::Regex;

fn uppercase(sql: &str) -> String {
    let result = Regex::new(r#"(?i)select\s+"#).unwrap().replace_all(sql, "SELECT ");
    let result = Regex::new(r#"(?i)distinct\s+"#).unwrap().replace_all(&result, "DISTINCT ");
    let result = Regex::new(r#"(?i)from\s+"#).unwrap().replace_all(&result, "FROM ");
    let result = Regex::new(r#"(?i)where\s+"#).unwrap().replace_all(&result, "WHERE ");
    let result = Regex::new(r#"(?i)left\s+join\s+"#).unwrap().replace_all(&result, "LEFT JOIN ");
    let result = Regex::new(r#"(?i)join\s+"#).unwrap().replace_all(&result, "JOIN ");
    let result = Regex::new(r#"(?i)update\s+"#).unwrap().replace_all(&result, "UPDATE ");
    let result = Regex::new(r#"(?i)\s+set\s+"#).unwrap().replace_all(&result, "SET ");
    let result = Regex::new(r#"(?i)insert\s+into\s+"#).unwrap().replace_all(&result, "INSERT INTO ");
    let result = Regex::new(r#"(?i)delete\s+"#).unwrap().replace_all(&result, "DELETE ");
    let result = Regex::new(r#"(?i)order\s+by\s+"#).unwrap().replace_all(&result, "ORDER BY ");
    result.into_owned()
}

fn unescape(sql: &str) -> String {
    let result = uppercase(sql);

    let re = Regex::new(r#"^[^"]*"\s*"#).unwrap();
    let result = re.replace_all(&result, "");

    let re = Regex::new(r#"\s*"\s*\+\s*"\s*"#).unwrap();
    let result = re.replace_all(&result, "\n");

    let re = Regex::new(r#"(?s)\\n|\s*".*$"#).unwrap();
    let result = re.replace_all(&result, "");

    let result = result + "\n;\n";
    //println!("{}", result);
    result.into_owned()
}

fn escape(sql: &str) -> String {
    let result = uppercase(sql);

    let re = Regex::new(r#"^\s*"#).unwrap();
    let result = re.replace_all(&result, "\"");

    let re = Regex::new(r#"(?s)\s*;?\s*$"#).unwrap();
    let result = re.replace_all(&result, "\"");

    let re = Regex::new(r#"\s*\n"#).unwrap();
    let result = re.replace_all(&result, "\\n\" +\n\"");

    //println!("{}", result);
    result.into_owned()
}

fn convert(sql: &str) -> String {
    if sql.contains("\"") {
        unescape(&sql)
    } else {
        escape(&sql)
    }
}

fn main() {
    ZKey.bind(|| {
        if LControlKey.is_pressed() && LShiftKey.is_pressed() {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

            let clipboard = ctx.get_contents().unwrap();
            println!("clipboard: {}", clipboard);

            let converted = convert(&clipboard);
            println!("converted: {}", converted);

            ctx.set_contents(converted).unwrap();
        }
    });
    handle_input_events();

    // if you run and quit this on linux have to run this, ignore for now
    // https://github.com/aweinstock314/rust-clipboard/issues/28
    //#[cfg(target_os = "linux")]
    //std::thread::sleep(std::time::Duration::from_millis(1000));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unescape_test() {
        let converted =
            r#"SELECT bob, tom
FROM charley
WHERE no > 1
ORDER BY 3
;
"#;
        assert_eq!(converted, convert(
            r#""select bob, tom\n" +
            "from charley\n" +
            "Where no > 1\n" +
            "order by 3""#));
        assert_eq!(converted, convert(
            r#"
            "select bob, tom\n" +
            "from charley\n" +
            "Where no > 1\n" +
            "order by 3"
            "#));
        assert_eq!(converted, convert(
            r#"
            @JdbcMapper.SQL("select bob, tom\n" +
            "from charley\n" +
            "Where no > 1\n" +
            "order by 3")
            "#));
        assert_eq!(converted, convert(
            r#"
            @SQL("select bob, tom\n" +
            "from charley\n" +
            "Where no > 1\n" +
            "order by 3")
            public Charley getCharley();
            "#));
        assert_eq!(converted, convert(
            r#"
            @SQL("select bob, tom " +
            "from charley " +
            "Where no > 1 " +
            "order by 3 ")
            public Charley getCharley();
            "#));
    }

    #[test]
    fn escape_test() {
        let converted =
            r#""SELECT bob, tom\n" +
"FROM charley\n" +
"WHERE no > 1\n" +
"ORDER BY 3""#;
        assert_eq!(converted, convert(
            r#"select bob, tom
from charley
Where no > 1
order by 3
;
"#));
        assert_eq!(converted, convert(
            r#"

            select bob, tom
from charley
Where no > 1
order by 3


"#));
    }
}