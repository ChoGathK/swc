use ast::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, FilePathMapping, Fold, FoldWith, SourceMap, Span, DUMMY_SP,
};
use swc_ecma_parser::{Parser, Session, SourceFileInput, Syntax};

/// Tracks used helper methods. (e.g. __extends)
#[derive(Default)]
pub struct Helpers {
    /// `_extends({}, b)`
    pub extends: AtomicBool,
    pub to_consumable_array: AtomicBool,
    /// `_classCallCheck`
    pub class_call_check: AtomicBool,
    /// `_inherits`
    pub inherits: AtomicBool,
    /// `_possibleConstructorReturn`
    pub possible_constructor_return: AtomicBool,
    ///`_createClass`
    pub create_class: AtomicBool,
    /// `_get`
    pub get: AtomicBool,
    /// `_instanceof`
    pub instance_of: AtomicBool,
    /// `_typeof`
    pub type_of: AtomicBool,
    /// `_taggedTemplateLiteral`
    pub tagged_template_literal: AtomicBool,
    /// `_defineProperty`
    pub define_property: AtomicBool,
    /// `_defineEnumerableProperties`
    pub define_enumerable_property: AtomicBool,
    /// `_set`
    pub set: AtomicBool,
    pub get_prototype_of: AtomicBool,
    pub throw: AtomicBool,
    pub async_to_generator: AtomicBool,
    pub object_without_properties: AtomicBool,
    pub object_spread: AtomicBool,
}

#[derive(Clone)]
pub struct InjectHelpers {
    pub cm: Lrc<SourceMap>,
    pub helpers: Arc<Helpers>,
}

impl InjectHelpers {
    fn mk_helpers(&self) -> Vec<Stmt> {
        lazy_static! {
            static ref CM: Lrc<SourceMap> = { Lrc::new(SourceMap::new(FilePathMapping::empty())) };
            static ref HANDLER: Handler =
                { Handler::with_tty_emitter(ColorConfig::Always, false, true, Some(CM.clone())) };
            static ref SESSION: Session<'static> = { Session { handler: &*HANDLER } };
        }

        let mut buf = vec![];

        macro_rules! add {
            ($name:tt, $b:expr) => {{
                lazy_static! {
                    static ref STMTS: Vec<Stmt> = {
                        let code = include_str!($name);
                        let fm = CM.new_source_file(
                            FileName::Custom(stringify!($name).into()),
                            code.into(),
                        );

                        let stmts =
                            Parser::new(*SESSION, Syntax::default(), SourceFileInput::from(&*fm))
                                .parse_script()
                                .map(|stmts| stmts.fold_with(&mut DropSpan))
                                .map_err(|mut e| {
                                    e.emit();
                                    ()
                                })
                                .unwrap();
                        stmts
                    };
                }

                let enable = $b.load(Ordering::Relaxed);
                if enable {
                    buf.extend_from_slice(&STMTS)
                }
            }};
        }

        add!("_extends.js", &self.helpers.extends);
        add!("_toConsumableArray.js", &self.helpers.to_consumable_array);
        add!("_classCallCheck.js", &self.helpers.class_call_check);
        add!("_inherits.js", &self.helpers.inherits);
        add!(
            "_possibleConstructorReturn.js",
            &self.helpers.possible_constructor_return
        );
        add!("_createClass.js", &self.helpers.create_class);
        add!("_get.js", &self.helpers.get);
        add!("_instanceof.js", &self.helpers.instance_of);
        add!("_typeof.js", &self.helpers.type_of);
        add!(
            "_taggedTemplateLiteral.js",
            &self.helpers.tagged_template_literal
        );
        add!("_defineProperty.js", &self.helpers.define_property);
        add!(
            "_defineEnumerableProperties.js",
            &self.helpers.define_enumerable_property
        );
        add!("_set.js", &self.helpers.set);
        add!("_getPrototypeOf.js", &self.helpers.get_prototype_of);
        add!("_throw.js", &self.helpers.throw);
        add!("_asyncToGenerator.js", &self.helpers.async_to_generator);
        add!(
            "_objectWithoutProperties.js",
            &self.helpers.object_without_properties
        );
        add!("_objectSpread.js", &self.helpers.object_spread);

        buf
    }
}

impl Fold<Module> for InjectHelpers {
    fn fold(&mut self, module: Module) -> Module {
        let body = self
            .mk_helpers()
            .into_iter()
            .map(ModuleItem::Stmt)
            .chain(module.body)
            .collect();

        Module { body, ..module }
    }
}

struct DropSpan;
impl Fold<Span> for DropSpan {
    fn fold(&mut self, _: Span) -> Span {
        DUMMY_SP
    }
}
