use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::fmt;

use super::{Export, Import, LocalNameResolver, ProcedureIndex, ProcedureName, ResolvedProcedure};
use crate::{
    ast::{AstSerdeOptions, Ident},
    diagnostics::{Report, SourceFile},
    sema::SemanticAnalysisError,
    ByteReader, ByteWriter, Deserializable, DeserializationError, LibraryNamespace, LibraryPath,
    Serializable, SourceSpan, Span, Spanned,
};

/// Represents the kind of a [Module].
///
/// The three different kinds have slightly different
/// rules on what syntax is allowed, as well as what
/// operations can be performed in the body of procedures
/// defined in the module. See the documentation for each
/// variant for a summary of these differences.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ModuleKind {
    /// A library is a simple container of code that must be
    /// included into an executable module to form a complete program.
    ///
    /// Library modules cannot use the `begin`..`end` syntax, which
    /// is used to define the entrypoint procedure for an executable.
    /// Aside from this, they are free to use all other MASM syntax.
    #[default]
    Library = 0,
    /// An executable is the root module of a program, and provides
    /// the entrypoint for executing that program.
    ///
    /// As the executable module is the root module, it may not export
    /// procedures for other modules to depend on, it may only import
    /// and call externally-defined procedures, or private locally-
    /// defined procedures.
    ///
    /// An executable module must contain a `begin`..`end` block.
    Executable = 1,
    /// A kernel is like a library module, but is special in a few ways:
    ///
    /// * Its code always executes in the root context, so it is stateful
    /// in a way that normal libraries cannot replicate. This can be used
    /// to provide core services that would otherwise not be possible to
    /// implement.
    ///
    /// * The procedures exported from the kernel may be the target of
    /// the `syscall` instruction, and in fact _must_ be called that way.
    ///
    /// * Kernels may not use `syscall` or `call` instructions internally
    Kernel = 2,
}
impl fmt::Display for ModuleKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Library => f.write_str("library"),
            Self::Executable => f.write_str("executable"),
            Self::Kernel => f.write_str("kernel"),
        }
    }
}
impl Serializable for ModuleKind {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8(*self as u8)
    }
}
impl Deserializable for ModuleKind {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        match source.read_u8()? {
            0 => Ok(Self::Library),
            1 => Ok(Self::Executable),
            2 => Ok(Self::Kernel),
            n => Err(DeserializationError::InvalidValue(format!("invalid module kind tag: {n}"))),
        }
    }
}

/// The abstract syntax tree for a single Miden Assembly module.
///
/// All module kinds share this AST representation, as they are
/// largely identical. However, the [ModuleKind] dictates how the
/// parsed module is semantically analyzed and validated.
#[derive(Clone)]
pub struct Module {
    /// The span covering the entire definition of this module
    span: SourceSpan,
    /// If available/known, the source contents from which this
    /// module was parsed. This is used to provide rich diagnostics
    /// output during semantic analysis.
    ///
    /// In cases where this file is not available, diagnostics will
    /// revert to a simple form with a helpful message, but without
    /// source code snippets.
    source_file: Option<Arc<SourceFile>>,
    /// The documentation associated with this module.
    ///
    /// Module documentation is provided in Miden Assembly as a
    /// documentation comment starting on the first line of the module.
    /// All other documentation comments are attached to the item
    /// the precede in the module body.
    docs: Option<Span<String>>,
    /// The fully-qualified path representing the name of this module
    path: LibraryPath,
    /// The kind of module this represents
    kind: ModuleKind,
    /// The imports defined in the module body
    pub(crate) imports: Vec<Import>,
    /// The procedures (defined or re-exported) in the module body
    ///
    /// NOTE: Despite the name, the procedures in this set are not
    /// necessarily exported, the individual procedure item must
    /// be checked to determine visibility
    pub(crate) procedures: Vec<Export>,
}

impl Spanned for Module {
    fn span(&self) -> SourceSpan {
        self.span
    }
}

/// Construction
impl Module {
    pub fn new(kind: ModuleKind, path: LibraryPath) -> Self {
        Self {
            span: Default::default(),
            source_file: None,
            docs: None,
            path,
            kind,
            imports: Default::default(),
            procedures: Default::default(),
        }
    }

    pub fn new_kernel() -> Self {
        Self::new(ModuleKind::Kernel, LibraryNamespace::Kernel.into())
    }

    pub fn new_executable() -> Self {
        Self::new(ModuleKind::Executable, LibraryNamespace::Exec.into())
    }

    pub fn with_source_file(mut self, source_file: Option<Arc<SourceFile>>) -> Self {
        self.source_file = source_file;
        self
    }

    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = span;
        self
    }

    pub fn set_source_file(&mut self, source_file: Arc<SourceFile>) {
        self.source_file = Some(source_file);
    }

    pub fn set_docs(&mut self, docs: Option<Span<String>>) {
        self.docs = docs;
    }

    pub fn set_span(&mut self, span: SourceSpan) {
        self.span = span;
    }

    /// Define a procedure, raise an error if the procedure conflicts with an existing definition
    pub fn define_procedure(&mut self, export: Export) -> Result<(), SemanticAnalysisError> {
        if self.is_kernel() && matches!(export, Export::Alias(_)) {
            return Err(SemanticAnalysisError::ImportToKernel {
                span: export.span(),
            });
        }
        if let Some(prev) = self.resolve(export.name()) {
            let prev_span = prev.span();
            Err(SemanticAnalysisError::SymbolConflict {
                span: export.span(),
                prev_span,
            })
        } else {
            self.procedures.push(export);
            Ok(())
        }
    }

    pub fn define_import(&mut self, import: Import) -> Result<(), SemanticAnalysisError> {
        if let Some(prev_import) = self.resolve_import(&import.name) {
            let prev_span = prev_import.span;
            return Err(SemanticAnalysisError::ImportConflict {
                span: import.span,
                prev_span,
            });
        }

        if let Some(prev_defined) = self.procedures.iter().find(|e| e.name().eq(&import.name)) {
            let prev_span = prev_defined.span();
            return Err(SemanticAnalysisError::SymbolConflict {
                span: import.span,
                prev_span,
            });
        }

        self.imports.push(import);

        Ok(())
    }
}

/// Parsing
impl Module {
    /// Parse a [Module], `name`, of the given [ModuleKind], from `path`.
    #[cfg(feature = "std")]
    pub fn parse_file<P>(name: LibraryPath, kind: ModuleKind, path: P) -> Result<Box<Self>, Report>
    where
        P: AsRef<std::path::Path>,
    {
        use crate::diagnostics::{IntoDiagnostic, WrapErr};
        let path = path.as_ref();
        let filename = path.to_string_lossy();
        let source = std::fs::read_to_string(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to parse module from '{filename}'"))?;
        let source_file = Arc::new(SourceFile::new(filename, source));
        Self::parse(name, kind, source_file)
    }

    /// Parse a [Module], `name`, of the given [ModuleKind], from `source`.
    pub fn parse_str(
        name: LibraryPath,
        kind: ModuleKind,
        source: impl ToString,
    ) -> Result<Box<Self>, Report> {
        let source = source.to_string();
        let source_file = Arc::new(SourceFile::new(name.path(), source));
        Self::parse(name, kind, source_file)
    }

    /// Parse a [Module], `name`, of the given [ModuleKind], from `source_file`.
    pub fn parse(
        name: LibraryPath,
        kind: ModuleKind,
        source_file: Arc<SourceFile>,
    ) -> Result<Box<Self>, Report> {
        let mut parser = crate::parser::ModuleParser::new(kind);
        parser.parse(name, source_file)
    }
}

/// Metadata
impl Module {
    pub fn source_file(&self) -> Option<Arc<SourceFile>> {
        self.source_file.clone()
    }

    pub fn name(&self) -> &str {
        self.path.last()
    }

    pub fn docs(&self) -> Option<Span<&str>> {
        self.docs.as_ref().map(|spanned| spanned.as_deref())
    }

    pub fn path(&self) -> &LibraryPath {
        &self.path
    }

    pub fn kind(&self) -> ModuleKind {
        self.kind
    }

    pub fn is_executable(&self) -> bool {
        matches!(self.kind, ModuleKind::Executable)
    }

    pub fn is_kernel(&self) -> bool {
        matches!(self.kind, ModuleKind::Kernel)
    }

    pub fn has_entrypoint(&self) -> bool {
        self.index_of(|p| p.is_main()).is_some()
    }

    pub fn namespace(&self) -> &LibraryNamespace {
        self.path.namespace()
    }

    /// Check if the module belongs to the provided namespace.
    pub fn is_in_namespace(&self, namespace: &LibraryNamespace) -> bool {
        self.path.namespace() == namespace
    }

    pub fn procedures(&self) -> core::slice::Iter<'_, Export> {
        self.procedures.iter()
    }

    pub fn procedures_mut(&mut self) -> core::slice::IterMut<'_, Export> {
        self.procedures.iter_mut()
    }

    pub fn imports(&self) -> core::slice::Iter<'_, Import> {
        self.imports.iter()
    }

    pub fn dependencies(&self) -> impl Iterator<Item = &LibraryNamespace> {
        self.import_paths().map(|import| import.namespace())
    }

    pub fn get(&self, index: ProcedureIndex) -> Option<&Export> {
        self.procedures.get(index.as_usize())
    }

    /// Get the [ProcedureIndex] for which `predicate` returns true in this module
    pub fn index_of<F>(&self, predicate: F) -> Option<ProcedureIndex>
    where
        F: FnMut(&Export) -> bool,
    {
        self.procedures.iter().position(predicate).map(ProcedureIndex::new)
    }

    /// Get the [ProcedureIndex] for corresponding to the procedure `name`
    /// in this module, _if_ the procedure is exported. Use [index_of] if
    /// you need more flexibility.
    pub fn index_of_name(&self, name: &ProcedureName) -> Option<ProcedureIndex> {
        self.index_of(|p| p.name() == name && p.visibility().is_exported())
    }

    /// Resolves `name` to a procedure within the context of this module
    pub fn resolve(&self, name: &ProcedureName) -> Option<ResolvedProcedure> {
        let index =
            self.procedures.iter().position(|p| p.name() == name).map(ProcedureIndex::new)?;
        match &self.procedures[index.as_usize()] {
            Export::Procedure(ref proc) => {
                Some(ResolvedProcedure::Local(Span::new(proc.name().span(), index)))
            }
            Export::Alias(ref alias) => Some(ResolvedProcedure::External(alias.target.clone())),
        }
    }

    /// Construct a search structure that can resolve procedure names local to this module
    pub fn resolver(&self) -> LocalNameResolver {
        LocalNameResolver::from_iter(self.procedures.iter().enumerate().map(|(i, p)| match p {
            Export::Procedure(ref p) => (
                p.name().clone(),
                ResolvedProcedure::Local(Span::new(p.name().span(), ProcedureIndex::new(i))),
            ),
            Export::Alias(ref p) => {
                (p.name().clone(), ResolvedProcedure::External(p.target.clone()))
            }
        }))
        .with_imports(
            self.imports
                .iter()
                .map(|import| (import.name.clone(), Span::new(import.span(), import.path.clone()))),
        )
    }

    /// Resolves `module_name` to an [Import] within the context of this module
    pub fn resolve_import(&self, module_name: &Ident) -> Option<&Import> {
        self.imports.iter().find(|import| &import.name == module_name)
    }

    /// Same as [resolve_import], but returns a mutable reference to the [Import]
    pub fn resolve_import_mut(&mut self, module_name: &Ident) -> Option<&mut Import> {
        self.imports.iter_mut().find(|import| &import.name == module_name)
    }

    /// Return an iterator over the paths of all imports in this module
    pub fn import_paths(&self) -> impl Iterator<Item = &LibraryPath> + '_ {
        self.imports.iter().map(|import| &import.path)
    }
}

impl core::ops::Index<ProcedureIndex> for Module {
    type Output = Export;

    #[inline]
    fn index(&self, index: ProcedureIndex) -> &Self::Output {
        &self.procedures[index.as_usize()]
    }
}
impl core::ops::IndexMut<ProcedureIndex> for Module {
    #[inline]
    fn index_mut(&mut self, index: ProcedureIndex) -> &mut Self::Output {
        &mut self.procedures[index.as_usize()]
    }
}
impl Eq for Module {}
impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.path == other.path
            && self.docs == other.docs
            && self.imports == other.imports
            && self.procedures == other.procedures
    }
}

/// Serialization
impl Module {
    /// Serialization this module to `target`, using `options`.
    pub fn write_into_with_options<W: ByteWriter>(&self, target: &mut W, options: AstSerdeOptions) {
        options.write_into(target);
        if options.debug_info {
            self.span.write_into(target);
            if let Some(source_file) = self.source_file.as_ref() {
                target.write_u8(1);
                let source_name = source_file.name();
                let source_bytes = source_file.inner().as_bytes();
                target.write_usize(source_name.as_bytes().len());
                target.write_bytes(source_name.as_bytes());
                target.write_usize(source_bytes.len());
                target.write_bytes(source_bytes);
            } else {
                target.write_u8(0);
            }
        }
        self.kind.write_into(target);
        self.path.write_into(target);
        if options.serialize_imports {
            target.write_usize(self.imports.len());
            for import in self.imports.iter() {
                import.write_into_with_options(target, options);
            }
        }
        target.write_usize(self.procedures.len());
        for export in self.procedures.iter() {
            export.write_into_with_options(target, options);
        }
    }

    /// Returns byte representation of this [Module].
    ///
    /// The serde options are serialized as header information for the purposes of deserialization.
    pub fn to_bytes(&self, options: AstSerdeOptions) -> Vec<u8> {
        let mut target = Vec::<u8>::with_capacity(256);
        self.write_into_with_options(&mut target, options);
        target
    }

    /// Returns a [Module] struct deserialized from the provided bytes.
    ///
    /// Assumes that the module was encoded using [write_into] or [write_into_with_options]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializationError> {
        let mut source = crate::SliceReader::new(bytes);
        Self::read_from(&mut source)
    }

    /// Writes this [Module] to the provided file path
    #[cfg(feature = "std")]
    pub fn write_to_file<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        use vm_core::utils::WriteAdapter;

        let path = path.as_ref();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        // NOTE: We're protecting against unwinds here due to i/o
        // errors that will get turned into panics if writing to
        // the underlying file fails. This is because ByteWriter does
        // not have fallible APIs, thus WriteAdapter has to panic
        // if writes fail. This could be fixed, but that has to happen
        // upstream in winterfell
        std::panic::catch_unwind(|| match std::fs::File::create(path) {
            Ok(ref mut file) => {
                let options = AstSerdeOptions {
                    serialize_imports: true,
                    debug_info: true,
                };
                let mut adapter = WriteAdapter::new(file);
                self.write_into_with_options(&mut adapter, options);
                Ok(())
            }
            Err(err) => Err(err),
        })
        .map_err(|p| {
            match p.downcast::<std::io::Error>() {
                // SAFETY: It is guaranteed to be safe to read Box<std::io::Error>
                Ok(err) => unsafe { core::ptr::read(&*err) },
                // Propagate unknown panics
                Err(err) => std::panic::resume_unwind(err),
            }
        })?
    }
}
impl Serializable for Module {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.write_into_with_options(target, AstSerdeOptions::new(true, true))
    }
}
impl Deserializable for Module {
    /// Deserialize a [Module] from `source`
    ///
    /// Assumes that the module was encoded using [write_into] or [write_into_with_options]
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let options = AstSerdeOptions::read_from(source)?;
        let (span, source_file) = if options.debug_info {
            let span = SourceSpan::read_from(source)?;
            match source.read_u8()? {
                0 => (span, None),
                1 => {
                    let nlen = source.read_usize()?;
                    let source_name = core::str::from_utf8(source.read_slice(nlen)?)
                        .map(|s| s.to_string())
                        .map_err(|e| DeserializationError::InvalidValue(e.to_string()))?;
                    let clen = source.read_usize()?;
                    let source_content = core::str::from_utf8(source.read_slice(clen)?)
                        .map_err(|e| DeserializationError::InvalidValue(e.to_string()))?;
                    let source_file =
                        Arc::new(SourceFile::new(source_name, source_content.to_string()));
                    (span, Some(source_file))
                }
                n => {
                    return Err(DeserializationError::InvalidValue(format!(
                        "invalid option tag: '{n}'"
                    )));
                }
            }
        } else {
            (SourceSpan::default(), None)
        };
        let kind = ModuleKind::read_from(source)?;
        let path = LibraryPath::read_from(source)?;
        let imports = if options.serialize_imports {
            let num_imports = source.read_usize()?;
            let mut imports = Vec::with_capacity(num_imports);
            for _ in 0..num_imports {
                let import = Import::read_from_with_options(source, options)?;
                imports.push(import);
            }
            imports
        } else {
            Vec::new()
        };
        let num_procedures = source.read_usize()?;
        let mut procedures = Vec::with_capacity(num_procedures);
        for _ in 0..num_procedures {
            let export = Export::read_from_with_options(source, options)?;
            procedures.push(export.with_source_file(source_file.clone()));
        }
        Ok(Self {
            span,
            source_file,
            docs: None,
            path,
            kind,
            imports,
            procedures,
        })
    }
}

/// Debug representation of this module
impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Module")
            .field("docs", &self.docs)
            .field("path", &self.path)
            .field("kind", &self.kind)
            .field("imports", &self.imports)
            .field("procedures", &self.procedures)
            .finish()
    }
}

/// Pretty-printed representation of this module as Miden Assembly text format
///
/// NOTE: Delegates to the [PrettyPrint] implementation internally
#[cfg(feature = "formatter")]
impl fmt::Display for Module {
    /// Writes this [Module] as formatted MASM code into the formatter.
    ///
    /// The formatted code puts each instruction on a separate line and preserves correct indentation
    /// for instruction blocks.
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::prettier::PrettyPrint;

        self.pretty_print(f)
    }
}

/// The pretty-printer for [Module]
#[cfg(feature = "formatter")]
impl crate::prettier::PrettyPrint for Module {
    fn render(&self) -> crate::prettier::Document {
        use crate::prettier::*;

        let mut doc = Document::Empty;
        if let Some(docs) = self.docs.as_ref() {
            let fragment =
                docs.lines().map(text).reduce(|acc, line| acc + nl() + text("#! ") + line);

            if let Some(fragment) = fragment {
                doc += fragment;
            }
        }

        for (i, import) in self.imports.iter().enumerate() {
            if i > 0 {
                doc += nl();
            }
            doc += import.render();
        }

        if !self.imports.is_empty() {
            doc += nl();
        }

        let mut export_index = 0;
        for export in self.procedures.iter() {
            if export.is_main() {
                continue;
            }
            if export_index > 0 {
                doc += nl();
            }
            doc += export.render();
            export_index += 1;
        }

        if let Some(main) = self.procedures().find(|p| p.is_main()) {
            doc += main.render();
        }

        doc
    }
}
