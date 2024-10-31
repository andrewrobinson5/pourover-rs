// template field configuration rules

// I need the following functionality:
//    store a pattern which matches the names and metadata of files that will be templated and formed into new files
//    a function that takes a list of files' information and determines which files match the above pattern ^
//    store some kind of container of the individual rules that will be used to derive the field data from the input files
//    a function that outputs a std::collections::HashMap<String, String> of field name, derived field data pairs

// still not sure what I'm gonna use for the user to be able to define the variables/fields
//  toml might be more than good enough
//  json might end up being too bloated/complex for what I'm looking for.
//    might look something like this:
//
//
//    [match]
//    path = "^content\/(?'year'\d{4})\/(?'month'\d{2})\/(?'filename'.*\.mdp)$"
//    template = "path/to/template"
//
//----- haha what if we let you write a lisp in here instead of whatever this made up syntax is
//    [meta-info]
//    output_filename = "./blog/{path.year}-{path.month}-{title}.html"
//    file = "(_EXEC cat {path})"
//    header = "(_REGEX (r'^po_header\\n(.*)\/po_header') {file})"
//
//    [fields]
//    title = "(_REGEX (r'title: (.*)\\n)' {header})"
//    body = "(_REGEX (r'\/po_header\\n(.*)$') {file})"
//
