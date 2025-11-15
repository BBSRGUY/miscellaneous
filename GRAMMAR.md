# Blang Grammar Specification (EBNF)

**Version**: 0.1
**Date**: 2025-01-15

---

## Notation

This grammar uses Extended Backus-Naur Form (EBNF) with the following conventions:

- `rule ::= definition` - Grammar rule definition
- `'literal'` - Terminal symbol (literal text)
- `"text"` - Terminal string
- `|` - Alternation (or)
- `()` - Grouping
- `[]` - Optional (zero or one)
- `{}` - Repetition (zero or more)
- `{1,}` - Repetition (one or more)
- `~` - Character range

---

## Lexical Grammar

### Whitespace and Comments

```ebnf
whitespace ::= (' ' | '\t' | '\r' | '\n')+

line_comment ::= '//' [^'\n']* '\n'

block_comment ::= '/*' ([^'*'] | '*'[^'/'])* '*/'

comment ::= line_comment | block_comment
```

### Identifiers

```ebnf
identifier ::= (letter | '_') (letter | digit | '_')*

letter ::= 'a'..'z' | 'A'..'Z' | unicode_letter

digit ::= '0'..'9'

unicode_letter ::= /* Any Unicode letter character */
```

### Keywords

```ebnf
keyword ::=
    'as' | 'break' | 'case' | 'catch' | 'component' | 'const' |
    'continue' | 'default' | 'do' | 'else' | 'enum' | 'export' |
    'false' | 'fn' | 'for' | 'if' | 'impl' | 'import' |
    'in' | 'interface' | 'job' | 'let' | 'loop' | 'match' |
    'module' | 'mut' | 'null' | 'parallel' | 'pub' | 'return' |
    'script' | 'self' | 'signal' | 'state' | 'step' | 'struct' |
    'style' | 'super' | 'trait' | 'true' | 'type' | 'typeof' |
    'unsafe' | 'use' | 'view' | 'while' | 'yield' |
    'actor' | 'channel' | 'depends_on' | 'receiver' | 'send' |
    'get' | 'set' | 'async' | 'await' | 'defer' |
    'on_mount' | 'on_update' | 'on_unmount' | 'computed' | 'props'
```

### Literals

```ebnf
literal ::=
    integer_literal |
    float_literal |
    boolean_literal |
    character_literal |
    string_literal

integer_literal ::=
    decimal_literal |
    hex_literal |
    octal_literal |
    binary_literal

decimal_literal ::= digit (digit | '_')* [integer_suffix]

hex_literal ::= '0x' hex_digit (hex_digit | '_')* [integer_suffix]

octal_literal ::= '0o' octal_digit (octal_digit | '_')* [integer_suffix]

binary_literal ::= '0b' binary_digit (binary_digit | '_')* [integer_suffix]

integer_suffix ::= 'i8' | 'i16' | 'i32' | 'i64' | 'u8' | 'u16' | 'u32' | 'u64'

hex_digit ::= digit | 'a'..'f' | 'A'..'F'

octal_digit ::= '0'..'7'

binary_digit ::= '0' | '1'

float_literal ::=
    digit (digit | '_')* '.' digit (digit | '_')* [exponent] [float_suffix] |
    digit (digit | '_')* exponent [float_suffix]

exponent ::= ('e' | 'E') ['+' | '-'] digit (digit | '_')*

float_suffix ::= 'f32' | 'f64'

boolean_literal ::= 'true' | 'false'

character_literal ::= "'" (char_content | escape_sequence) "'"

char_content ::= [^'\'' '\\' '\n' '\r' '\t']

string_literal ::=
    simple_string |
    raw_string |
    template_string

simple_string ::= '"' {string_content | escape_sequence} '"'

string_content ::= [^'"' '\\' '\n']

raw_string ::= 'r' raw_string_delimiter string_body raw_string_delimiter

raw_string_delimiter ::= '#'*

template_string ::= '`' {template_content | template_interpolation} '`'

template_content ::= [^'`' '$' '\\'] | '\\' .

template_interpolation ::= '${' expression '}'

escape_sequence ::=
    '\\' ('n' | 'r' | 't' | '\\' | '\'' | '"' | '0') |
    '\\x' hex_digit hex_digit |
    '\\u{' hex_digit{1,6} '}'
```

### Operators and Punctuation

```ebnf
operator ::=
    '+' | '-' | '*' | '/' | '%' |
    '==' | '!=' | '<' | '<=' | '>' | '>=' |
    '&&' | '||' | '!' |
    '&' | '|' | '^' | '~' | '<<' | '>>' |
    '=' | '+=' | '-=' | '*=' | '/=' | '%=' |
    '&=' | '|=' | '^=' | '<<=' | '>>=' |
    '->' | '=>' | '::' | '.' | '..' | '..=' |
    '@' | '#' | '$'

punctuation ::=
    '(' | ')' | '[' | ']' | '{' | '}' |
    ',' | ';' | ':'
```

---

## Syntactic Grammar

### Source File

```ebnf
source_file ::=
    module_file |
    component_file |
    script_file

module_file ::= {attribute} module_declaration

component_file ::= {attribute} component_declaration

script_file ::= {attribute} script_declaration
```

### Attributes

```ebnf
attribute ::= '#[' attribute_content ']'

attribute_content ::=
    identifier ['(' attribute_args ')']

attribute_args ::=
    attribute_arg {',' attribute_arg} [',']

attribute_arg ::=
    identifier ['=' literal]
```

### Module Declaration

```ebnf
module_declaration ::=
    ['pub'] 'module' identifier '{' {module_item} '}'

module_item ::=
    module_declaration |
    use_declaration |
    function_declaration |
    struct_declaration |
    enum_declaration |
    trait_declaration |
    impl_declaration |
    type_alias_declaration |
    const_declaration |
    static_declaration
```

### Use Declaration

```ebnf
use_declaration ::=
    'use' use_path ';'

use_path ::=
    identifier {'::' identifier} |
    identifier '::' '{' use_list '}' |
    identifier '::' '*' |
    use_path 'as' identifier

use_list ::=
    use_path {',' use_path} [',']
```

### Component Declaration

```ebnf
component_declaration ::=
    'component' identifier [generic_params] '{' {component_item} '}'

component_item ::=
    component_props |
    component_state |
    component_computed |
    component_view |
    component_style |
    component_lifecycle |
    function_declaration

component_props ::=
    'props' '{' {prop_declaration} '}'

prop_declaration ::=
    identifier ':' type_expression ['=' expression] ','

component_state ::=
    'state' '{' {state_declaration} '}'

state_declaration ::=
    identifier ':' type_expression ['=' expression] ';'

component_computed ::=
    'computed' '{' {computed_declaration} '}'

computed_declaration ::=
    identifier ':' type_expression '=' block_expression ';'

component_view ::=
    'view' '{' view_content '}'

view_content ::=
    html_element |
    html_text |
    html_interpolation |
    html_conditional |
    html_loop |
    '{' view_content '}' |
    view_content view_content

html_element ::=
    '<' identifier {html_attribute} ['/'] '>' |
    '<' identifier {html_attribute} '>' {view_content} '</' identifier '>'

html_attribute ::=
    identifier ['=' (string_literal | '{' expression '}')] |
    '@' identifier '=' '{' expression '}'

html_text ::= [^'<' '{' '}']+

html_interpolation ::= '{' expression '}'

html_conditional ::=
    '{' 'if' expression '{' view_content '}' ['else' '{' view_content '}'] '}'

html_loop ::=
    '{' 'for' pattern 'in' expression '{' view_content '}' '}'

component_style ::=
    'style' '{' css_content '}'

css_content ::= /* Standard CSS syntax */

component_lifecycle ::=
    'on_mount' block_expression |
    'on_update' block_expression |
    'on_unmount' block_expression
```

### Script Declaration

```ebnf
script_declaration ::=
    'script' identifier '{' {script_item} '}'

script_item ::=
    script_config |
    script_schedule |
    script_state |
    job_declaration |
    function_declaration

script_config ::=
    'config' '{' {config_option} '}'

config_option ::=
    identifier ':' literal ','

script_schedule ::=
    'schedule' '{' {schedule_option} '}'

schedule_option ::=
    identifier ':' literal ','

script_state ::=
    'state' '{' {state_declaration} '}'

job_declaration ::=
    'job' identifier [job_dependencies] '{' {job_item} '}'

job_dependencies ::=
    'depends_on' '(' identifier {',' identifier} [','] ')'

job_item ::=
    step_declaration |
    parallel_block

step_declaration ::=
    'step' identifier [step_config] block_expression

step_config ::=
    '{' {step_option ','} '}'

step_option ::=
    identifier ':' (literal | identifier)

parallel_block ::=
    'parallel' '{' {step_declaration} '}'
```

### Function Declaration

```ebnf
function_declaration ::=
    {attribute} function_signature block_expression

function_signature ::=
    ['pub'] ['unsafe'] 'fn' identifier [generic_params] '(' [parameters] ')' ['->' type_expression]

parameters ::=
    parameter {',' parameter} [',']

parameter ::=
    ['mut'] identifier ':' type_expression

generic_params ::=
    '<' generic_param {',' generic_param} [','] '>'

generic_param ::=
    identifier [':' trait_bounds]

trait_bounds ::=
    trait_bound {'+' trait_bound}

trait_bound ::=
    identifier [generic_args]
```

### Type Declarations

```ebnf
struct_declaration ::=
    ['pub'] 'struct' identifier [generic_params] struct_body

struct_body ::=
    '{' {struct_field} '}' |
    '(' {tuple_field} ')' ';' |
    ';'

struct_field ::=
    ['pub'] identifier ':' type_expression ','

tuple_field ::=
    ['pub'] type_expression ','

enum_declaration ::=
    ['pub'] 'enum' identifier [generic_params] '{' {enum_variant} '}'

enum_variant ::=
    identifier [enum_variant_body] ','

enum_variant_body ::=
    '(' {type_expression ','} ')' |
    '{' {struct_field} '}'

trait_declaration ::=
    ['pub'] 'trait' identifier [generic_params] [':' trait_bounds] '{' {trait_item} '}'

trait_item ::=
    associated_type |
    trait_function

associated_type ::=
    'type' identifier [':' trait_bounds] ';'

trait_function ::=
    function_signature (';' | block_expression)

impl_declaration ::=
    'impl' [generic_params] impl_target [impl_for] '{' {impl_item} '}'

impl_target ::=
    type_expression

impl_for ::=
    'for' type_expression

impl_item ::=
    associated_type_impl |
    function_declaration

associated_type_impl ::=
    'type' identifier '=' type_expression ';'

type_alias_declaration ::=
    ['pub'] 'type' identifier [generic_params] '=' type_expression ';'

const_declaration ::=
    ['pub'] 'const' identifier ':' type_expression '=' expression ';'

static_declaration ::=
    ['pub'] 'static' ['mut'] identifier ':' type_expression '=' expression ';'
```

### Type Expressions

```ebnf
type_expression ::=
    primitive_type |
    path_type |
    reference_type |
    pointer_type |
    array_type |
    slice_type |
    tuple_type |
    function_type |
    '(' type_expression ')' |
    'never'

primitive_type ::=
    'i8' | 'i16' | 'i32' | 'i64' |
    'u8' | 'u16' | 'u32' | 'u64' |
    'f32' | 'f64' |
    'bool' | 'char' | 'str' | '()'

path_type ::=
    identifier {'::' identifier} [generic_args]

generic_args ::=
    '<' type_expression {',' type_expression} [','] '>'

reference_type ::=
    '&' ['mut'] type_expression

pointer_type ::=
    '*' ('const' | 'mut') type_expression

array_type ::=
    '[' type_expression ';' expression ']'

slice_type ::=
    '[' type_expression ']'

tuple_type ::=
    '(' [type_expression {',' type_expression} [',']] ')'

function_type ::=
    'fn' '(' [type_expression {',' type_expression} [',']] ')' ['->' type_expression]
```

### Statements

```ebnf
statement ::=
    let_statement |
    expression_statement |
    item_statement

let_statement ::=
    'let' ['mut'] pattern [':' type_expression] ['=' expression] ';'

expression_statement ::=
    expression_without_block ';' |
    expression_with_block [';']

item_statement ::=
    function_declaration |
    struct_declaration |
    enum_declaration |
    trait_declaration |
    impl_declaration |
    type_alias_declaration |
    const_declaration |
    static_declaration
```

### Patterns

```ebnf
pattern ::=
    literal_pattern |
    identifier_pattern |
    wildcard_pattern |
    rest_pattern |
    reference_pattern |
    struct_pattern |
    tuple_pattern |
    enum_pattern |
    or_pattern |
    '(' pattern ')'

literal_pattern ::= literal

identifier_pattern ::=
    ['mut'] identifier ['@' pattern]

wildcard_pattern ::= '_'

rest_pattern ::= '..'

reference_pattern ::=
    '&' ['mut'] pattern

struct_pattern ::=
    path_type '{' {field_pattern} ['..' ] '}'

field_pattern ::=
    identifier [':' pattern] ','

tuple_pattern ::=
    path_type '(' {pattern ','} [rest_pattern] ')'

enum_pattern ::=
    path_type ['(' {pattern ','} ')' | '{' {field_pattern} '}']

or_pattern ::=
    pattern '|' pattern
```

### Expressions

```ebnf
expression ::=
    expression_without_block |
    expression_with_block

expression_without_block ::=
    literal_expression |
    path_expression |
    operator_expression |
    call_expression |
    method_call_expression |
    field_expression |
    index_expression |
    range_expression |
    closure_expression |
    return_expression |
    break_expression |
    continue_expression |
    array_expression |
    tuple_expression |
    struct_expression |
    await_expression |
    cast_expression |
    reference_expression |
    dereference_expression

expression_with_block ::=
    block_expression |
    if_expression |
    match_expression |
    loop_expression |
    while_expression |
    for_expression |
    unsafe_block

literal_expression ::= literal

path_expression ::=
    identifier {'::' identifier}

operator_expression ::=
    expression binary_operator expression |
    unary_operator expression

binary_operator ::=
    '+' | '-' | '*' | '/' | '%' |
    '==' | '!=' | '<' | '<=' | '>' | '>=' |
    '&&' | '||' |
    '&' | '|' | '^' | '<<' | '>>' |
    '=' | '+=' | '-=' | '*=' | '/=' | '%=' |
    '&=' | '|=' | '^=' | '<<=' | '>>='

unary_operator ::=
    '-' | '!' | '~'

call_expression ::=
    expression '(' [arguments] ')'

arguments ::=
    expression {',' expression} [',']

method_call_expression ::=
    expression '.' identifier [generic_args] '(' [arguments] ')'

field_expression ::=
    expression '.' (identifier | integer_literal)

index_expression ::=
    expression '[' expression ']'

range_expression ::=
    expression? '..' expression? |
    expression? '..=' expression

closure_expression ::=
    '|' [closure_parameters] '|' (expression | block_expression)

closure_parameters ::=
    closure_parameter {',' closure_parameter} [',']

closure_parameter ::=
    pattern [':' type_expression]

return_expression ::=
    'return' [expression]

break_expression ::=
    'break' [expression]

continue_expression ::=
    'continue'

array_expression ::=
    '[' [array_elements] ']'

array_elements ::=
    expression {',' expression} [','] |
    expression ';' expression

tuple_expression ::=
    '(' [expression {',' expression} [',']] ')'

struct_expression ::=
    path_type '{' [struct_expr_fields] '}'

struct_expr_fields ::=
    struct_expr_field {',' struct_expr_field} [','] ['..' expression]

struct_expr_field ::=
    identifier [':' expression]

await_expression ::=
    expression '.' 'await'

cast_expression ::=
    expression 'as' type_expression

reference_expression ::=
    '&' ['mut'] expression

dereference_expression ::=
    '*' expression

block_expression ::=
    '{' {statement} [expression] '}'

if_expression ::=
    'if' expression block_expression ['else' (if_expression | block_expression)]

match_expression ::=
    'match' expression '{' {match_arm} '}'

match_arm ::=
    pattern [match_guard] '=>' (expression | block_expression) ','

match_guard ::=
    'if' expression

loop_expression ::=
    'loop' block_expression

while_expression ::=
    'while' expression block_expression

for_expression ::=
    'for' pattern 'in' expression block_expression

unsafe_block ::=
    'unsafe' (block_expression | wasm_block)

wasm_block ::=
    'wasm!' '{' wasm_instructions '}'

wasm_instructions ::=
    /* WebAssembly text format instructions */
```

### Actor Declaration

```ebnf
actor_declaration ::=
    'actor' identifier [generic_params] '{' {actor_item} '}'

actor_item ::=
    actor_state |
    receiver_declaration |
    function_declaration

actor_state ::=
    'state' '{' {state_declaration} '}'

receiver_declaration ::=
    'receiver' identifier '(' [parameters] ')' ['->' type_expression] block_expression
```

### Signal Expressions

```ebnf
signal_expression ::=
    'signal' '(' expression ')'

effect_expression ::=
    'effect' '(' closure_expression ')'

memo_expression ::=
    'memo' '(' closure_expression ')'

resource_expression ::=
    'resource' '(' closure_expression ',' closure_expression ')'
```

### Operator Precedence

From highest to lowest:

1. Field access, method calls, indexing: `.`, `[]`, `()`
2. Unary operators: `-`, `!`, `~`, `*`, `&`
3. Type cast: `as`
4. Multiplication, division, remainder: `*`, `/`, `%`
5. Addition, subtraction: `+`, `-`
6. Bit shifts: `<<`, `>>`
7. Bitwise AND: `&`
8. Bitwise XOR: `^`
9. Bitwise OR: `|`
10. Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
11. Logical AND: `&&`
12. Logical OR: `||`
13. Range: `..`, `..=`
14. Assignment: `=`, `+=`, `-=`, etc.

### Associativity

- Most binary operators are left-associative
- Assignment operators are right-associative
- Comparison operators are non-associative (cannot chain without parentheses)

---

## Context-Free Properties

### Scoping Rules

1. **Block Scope**: Variables declared with `let` are scoped to their containing block
2. **Function Scope**: Functions create a new scope
3. **Module Scope**: Items in a module are scoped to that module
4. **Component Scope**: Component state, props, and computed values are scoped to the component
5. **Script Scope**: Jobs and steps are scoped to their containing script

### Name Resolution

1. Local variables shadow outer scopes
2. Items must be declared before use (except for mutual recursion in functions)
3. Imports are resolved before other names
4. Type names and value names are in separate namespaces

### Type System Properties

1. **Type Inference**: Types are inferred bidirectionally
2. **Generics**: Support for parametric polymorphism with trait bounds
3. **Lifetime Elision**: Simple lifetime rules allow omitting lifetime annotations in common cases
4. **Trait Coherence**: One impl per type-trait pair

---

## Macro Grammar (Future)

```ebnf
macro_invocation ::=
    identifier '!' token_tree

macro_definition ::=
    'macro' identifier macro_rules

macro_rules ::=
    '{' {macro_rule ','} '}'

macro_rule ::=
    '(' matcher ')' '=>' transcriber

matcher ::=
    /* Pattern matching on token trees */

transcriber ::=
    /* Token tree generation */

token_tree ::=
    token |
    '(' {token_tree} ')' |
    '[' {token_tree} ']' |
    '{' {token_tree} '}'
```

---

## Standard Attributes

```ebnf
inline_attribute ::= '#[inline]' | '#[inline(always)]' | '#[inline(never)]'

derive_attribute ::= '#[derive(' identifier {',' identifier} ')]'

test_attribute ::= '#[test]'

cfg_attribute ::= '#[cfg(' cfg_predicate ')]'

deprecated_attribute ::= '#[deprecated]' | '#[deprecated(since = "version", note = "message")]'

export_attribute ::= '#[export]'

no_mangle_attribute ::= '#[no_mangle]'
```

---

## Examples

### Module Example

```blang
module math {
  pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
      return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
  }

  pub struct Point {
    pub x: f64,
    pub y: f64,
  }

  impl Point {
    pub fn distance(&self) -> f64 {
      (self.x * self.x + self.y * self.y).sqrt()
    }
  }
}
```

### Component Example

```blang
component Counter {
  state {
    count: Signal<i32> = signal(0);
  }

  view {
    <div class="counter">
      <button @click={decrement}>-</button>
      <span>{count}</span>
      <button @click={increment}>+</button>
    </div>
  }

  style {
    .counter {
      display: flex;
      gap: 1rem;
    }
  }

  fn increment() {
    count.update(|c| c + 1);
  }

  fn decrement() {
    count.update(|c| c - 1);
  }
}
```

### Script Example

```blang
script Pipeline {
  job extract {
    step fetch {
      retry: 3,
      timeout: 30s,

      let data = http.get("/api/data");
      return data;
    }
  }

  job transform depends_on(extract) {
    parallel {
      step process_a {
        return process(extract.fetch.result, "a");
      }

      step process_b {
        return process(extract.fetch.result, "b");
      }
    }
  }

  job load depends_on(transform) {
    step save {
      let a = transform.process_a.result;
      let b = transform.process_b.result;
      db.save(merge(a, b));
    }
  }
}
```

---

## Grammar Notes

1. **Whitespace**: Whitespace and comments are ignored except where they separate tokens
2. **Semicolon Insertion**: No automatic semicolon insertion; semicolons are explicit
3. **Expression vs Statement**: Expressions can be used where statements are expected
4. **Block Values**: Blocks are expressions and evaluate to their final expression
5. **Trailing Commas**: Allowed in all comma-separated lists

---

## Reserved Syntax

The following syntax is reserved for future use:

- `@annotation` syntax (beyond event handlers)
- `$variable` syntax (beyond template strings)
- `#identifier` syntax (beyond attributes)
- Backtick identifiers: `` `identifier` ``
- Double colon for paths: Already used
- Pipeline operator: `|>`
- Composition operator: `>>`

---

## Version History

- **v0.1** (2025-01-15): Initial grammar specification

---

This grammar is designed to be:

1. **Unambiguous**: No shift-reduce or reduce-reduce conflicts
2. **Complete**: Covers all language features
3. **Extensible**: Easy to add new features
4. **Parser-Friendly**: Suitable for LL(k) or LR parsers
5. **Human-Readable**: Clear and understandable

For implementation details, see the Language Specification (LANGUAGE_SPEC.md) and Development Roadmap (ROADMAP.md).
