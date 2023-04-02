use async_trait::async_trait;

use crate::{impl_langserver_commands, socket::SendToSocket, socket::SocketAbstraction};

use super::{LSReq, LangServer, LangServerError};

#[derive(Debug)]
pub struct TsServer {
    socket: SocketAbstraction,
}

#[async_trait]
impl LangServer for TsServer {
    async fn make(server_path: &str) -> Result<Self, LangServerError> {
        let args = ["npm", "--prefix", server_path, "start"];
        let socket = SocketAbstraction::spawn_server("typescript", &args, true)
            .await
            .map_err(|_| LangServerError::ProcessSpawn)?;
        Ok(Self { socket })
    }

    async fn type_check(&self, code: &str) -> Result<bool, LangServerError> {
        // for typescript, we use the language server for typechecking
        let req = LSReq {
            cmd: "typecheck".to_string(),
            text: base64::encode(code),
        };
        let resp = self
            .socket
            .send_req(serde_json::to_value(&req).unwrap())
            .await?;

        let errors: usize = resp["errors"].as_u64().unwrap() as usize;
        Ok(errors == 0)
    }

    fn any_type(&self) -> String {
        "any".to_string()
    }

    fn get_type_parser(&self) -> Option<Box<dyn Fn(&str) -> Option<String> + Sync + Send>> {
        #[cfg(feature = "tsparser")]
        {
            Some(Box::new(ts_parse_type))
        }
        #[cfg(not(feature = "tsparser"))]
        {
            None
        }
    }
}

// implement the LangServerCommands trait
impl_langserver_commands!(TsServer);

#[cfg(feature = "tsparser")]
/// Parses the given input and extracts the type generated by a model.
/// Code modified, but originally from: https://github.com/nuprl/TypeWeaver
pub fn ts_parse_type(input: &str) -> Option<String> {
    use swc_common::sync::Lrc;
    use swc_common::{FileName, SourceMap, Spanned};
    use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
    let input = input.trim().to_string();

    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.new_source_file(FileName::Anon, input.clone());

    let string_input = StringInput::from(&*fm);
    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        string_input,
        None,
    );

    let mut parser = Parser::new_from(lexer);
    match parser.parse_type() {
        Err(_) => None,
        Ok(typ) => {
            if !parser.take_errors().is_empty() {
                return None;
            }
            let hi: usize = typ.span_hi().0.try_into().unwrap();
            let input_prefix = &fm.src[..hi - 1];

            // there are some edge cases. for instance, this.blah gets parsed as a
            // `this`. we should only parse as `this` if it's the only thing in the
            // input.
            if input_prefix.trim() == "this" && input.contains("this.") {
                return None;
            }
            Some(input_prefix.trim().to_string())
        }
    }
}
