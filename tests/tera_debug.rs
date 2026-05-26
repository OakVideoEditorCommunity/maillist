use tera::{Context, Tera};

#[test]
fn test_tera_render() {
    let mut ctx = Context::new();
    ctx.insert("list_name", "Test List");
    ctx.insert("subscriber_name", "Alice");
    let tmpl = "欢迎加入 {{ list_name }}\n您好 {{ subscriber_name | default(value=\"\") }},";
    match Tera::one_off(tmpl, &ctx, false) {
        Ok(s) => println!("OK: {}", s),
        Err(e) => panic!("ERR: {}", e),
    }
}
