use memchr::{memchr, memchr_iter};
use nom::{
    branch::{alt},
    bytes::complete::{tag},
    character::complete::{one_of, alpha1, multispace0, digit1, char},
    error::*,
    multi::{fold_many1, fold_many0, many0_count, separated_list0, separated_list1, many0, many1},
    combinator::{map, recognize, opt},
    sequence::{delimited, pair, preceded, tuple, terminated},
};

enum Parse {
    LogicalLiteral(LogicalLiteral),
}

pub type IResult<'a, U> = nom::IResult<&'a str, U, nom::error::VerboseError<&'a str>>;

fn build_err<'a, U>(s: &'a str, msg: &'static str) -> IResult<'a, U> {
    Err(nom::Err::Error(
        VerboseError {
            errors: vec![(s, VerboseErrorKind::Context(msg))]
        }))
}

/// Returns a parser which runs `p` then consumes all whitespace
fn ws<'a, U, F>(p: F) -> impl FnMut(&'a str) -> IResult<'a, U>
    where F: FnMut(&'a str) -> IResult<'a, U>
{
    terminated(p, multispace0)
}


////////////////////////////////////////////////////////////////////////////////

// 124
fn digit(s: &str) -> IResult<char> {
    one_of("0123456789")(s)
}

// 125
fn digits(s: &str) -> IResult<usize> {
    map(digit1, |v: &str| v.parse().unwrap())(s)
}

// 127
fn hex_digit(s: &str) -> IResult<char> {
    alt((digit, one_of("abcdef")))(s)
}

// 126
fn encoded_character(s: &str) -> IResult<char> {
    map(recognize(tuple((octet, octet, octet, octet))),
        |v| std::char::from_u32(u32::from_str_radix(v, 16).unwrap()).unwrap())
        (s)
}

// 128
fn letter(s: &str) -> IResult<char> {
    one_of("abcdefghijklmnopqrstuvwxyz")(s)
}

// 132
fn not_paren_star_quote_special(s: &str) -> IResult<char> {
    one_of("!\"#$%&+,-./:;<=>?@[\\]^_‘{|}~")(s)
}

// 134
fn not_quote(s: &str) -> IResult<char> {
    alt((not_paren_star_quote_special, letter, digit, one_of("()*")))(s)
}

// 136
fn octet(s: &str) -> IResult<&str> {
    recognize(pair(hex_digit, hex_digit))(s)
}

// 139
fn binary_literal(s: &str) -> IResult<usize> {
    let bits = fold_many1(alt((char('0'), char('1'))), 0,
        |mut acc, item| acc * 2 + item.to_digit(10).unwrap() as usize);
    preceded(char('%'), bits)(s)
}

// 140
fn encoded_string_literal(s: &str) -> IResult<String> {
    delimited(
        char('"'),
        fold_many0(encoded_character, String::new(),
            |mut s: String, c: char| { s.push(c); s }),
        char('"'))(s)
}

// 141
fn integer_literal(s: &str) -> IResult<usize> {
    digits(s)
}

// 142
fn real_literal(s: &str) -> IResult<f64> {
    match fast_float::parse_partial::<f64, _>(s) {
        Err(_) => build_err(s, "Could not parse float"),
        Ok((x, n)) => Ok((&s[n..], x)),
    }
}

// 143
struct SimpleId<'a>(&'a str);
fn simple_id(s: &str) -> IResult<SimpleId> {
    map(pair(
            alpha1,
            many0_count(alt((letter, digit, char('_'))))),
        |(_c, i)| SimpleId(&s[1..(i + 1)]))(s)
}

// 144
fn simple_string_literal(s: &str) -> IResult<String> {
    let f = alt((
        map(tag("''"), |_| '\''),
        not_paren_star_quote_special,
        letter,
        digit,
        one_of("()*")
    ));
    delimited(
            char('\''),
            fold_many0(f, String::new(), |mut s, c| { s.push(c); s }),
            char('\''))(s)
}

// 154
struct FunctionRef<'a>(FunctionId<'a>);
fn function_ref(s: &str) -> IResult<FunctionRef> {
    map(function_id, FunctionRef)(s)
}

// 155
struct ParameterRef<'a>(ParameterId<'a>);
fn parameter_ref(s: &str) -> IResult<ParameterRef> {
    map(parameter_id, ParameterRef)(s)
}

// 168
enum AddLikeOp { Add, Sub, Or, Xor }
fn add_like_op(s: &str) -> IResult<AddLikeOp> {
    use AddLikeOp::*;
    alt((
        map(char('+'),  |_| Add),
        map(char('-'),  |_| Sub),
        map(tag("or"),  |_| Or),
        map(tag("xor"), |_| Xor),
    ))(s)
}

// 150
struct AttributeRef<'a>(AttributeId<'a>);
fn attribute_ref(s: &str) -> IResult<AttributeRef> {
    map(attribute_id, AttributeRef)(s)
}

// 151
struct ConstantRef<'a>(ConstantId<'a>);
fn constant_ref(s: &str) -> IResult<ConstantRef> {
    map(constant_id, ConstantRef)(s)
}

// 152
struct EntityRef<'a>(EntityId<'a>);
fn entity_ref(s: &str) -> IResult<EntityRef> {
    map(entity_id, EntityRef)(s)
}

// 153
struct EnumerationRef<'a>(EnumerationId<'a>);
fn enumeration_ref(s: &str) -> IResult<EnumerationRef> {
    map(enumeration_id, EnumerationRef)(s)
}

// 162
struct TypeRef<'a>(TypeId<'a>);
fn type_ref(s: &str) -> IResult<TypeRef> {
    map(type_id, TypeRef)(s)
}

// 163
struct VariableRef<'a>(VariableId<'a>);
fn variable_ref(s: &str) -> IResult<VariableRef> {
    map(variable_id, VariableRef)(s)
}

// 167
struct ActualParameterList<'a>(Vec<Parameter<'a>>);
fn actual_parameter_list(s: &str) -> IResult<ActualParameterList> {
    map(delimited(
            ws(char('(')),
            separated_list1(ws(char(',')), parameter),
            ws(char(')'))),
        ActualParameterList)(s)
}

// 169
struct AggregateInitializer<'a>(Vec<Element<'a>>);
fn aggregate_initializer(s: &str) -> IResult<AggregateInitializer> {
    map(delimited(
            ws(char('[')),
            separated_list0(ws(char(',')), ws(element)),
            ws(char(']'))),
        AggregateInitializer)(s)
}

// 170
struct AggregateSource<'a>(SimpleExpression<'a>);
fn aggregate_source(s: &str) -> IResult<AggregateSource> {
    map(simple_expression, AggregateSource)(s)
}

// 172
enum AggregationTypes<'a> {
    Array(ArrayType<'a>),
    Bag(BagType<'a>),
    List(ListType<'a>),
    Set(SetType<'a>),
}
fn aggregation_types(s: &str) -> IResult<AggregationTypes> {
    use AggregationTypes::*;
    alt((
        map(array_type, Array),
        map(bag_type, Bag),
        map(list_type, List),
        map(set_type, Set),
    ))(s)
}

// 175
struct ArrayType<'a> {
    bounds: BoundSpec<'a>,
    optional: bool,
    unique: bool,
    instantiable_type: Box<InstantiableType<'a>>,
}
fn array_type(s: &str) -> IResult<ArrayType> {
    map(tuple((
        ws(tag("array")),
        ws(bound_spec),
        ws(tag("of")),
        ws(opt(tag("optional"))),
        ws(opt(tag("unique"))),
        ws(instantiable_type),
    )),
    |(_, b, _, opt, uniq, t)| ArrayType {
        bounds: b,
        optional: opt.is_some(),
        unique: uniq.is_some(),
        instantiable_type: Box::new(t),
    })(s)
}

// 178
struct AttributeId<'a>(SimpleId<'a>);
fn attribute_id(s: &str) -> IResult<AttributeId> {
    map(simple_id, AttributeId)(s)
}

// 179
struct AttributeQualifier<'a>(AttributeRef<'a>);
fn attribute_qualifier(s: &str) -> IResult<AttributeQualifier> {
    map(preceded(ws(char('.')), attribute_ref), AttributeQualifier)(s)
}

// 180
struct BagType<'a>(Option<BoundSpec<'a>>, Box<InstantiableType<'a>>);
fn bag_type(s: &str) -> IResult<BagType> {
    map(tuple((
            ws(tag("BAG")),
            ws(opt(bound_spec)),
            ws(tag("OF")),
            ws(instantiable_type)
        )), |(_, b, _, t)| BagType(b, Box::new(t)))
        (s)
}

// 183
struct Bound1<'a>(NumericalExpression<'a>);
fn bound_1(s: &str) -> IResult<Bound1> {
    map(numerical_expression, Bound1)(s)
}

// 184
struct Bound2<'a>(NumericalExpression<'a>);
fn bound_2(s: &str) -> IResult<Bound2> {
    map(numerical_expression, Bound2)(s)
}

// 185
struct BoundSpec<'a>(Bound1<'a>, Bound2<'a>);
fn bound_spec(s: &str) -> IResult<BoundSpec> {
    map(tuple((
        ws(char('[')),
        ws(bound_1),
        ws(char(':')),
        ws(bound_2),
        ws(char(']')),
    )), |(_, b1, _, b2, _)| BoundSpec(b1, b2))(s)
}

// 186
enum BuiltInConstant { ConstE, Pi, Self_, Indeterminant }
fn built_in_constant(s: &str) -> IResult<BuiltInConstant> {
    use BuiltInConstant::*;
    alt((
        map(tag("const_e"), |_| ConstE),
        map(tag("pi"),      |_| Pi),
        map(tag("self"),    |_| Self_),
        map(char('?'),      |_| Indeterminant),
    ))(s)
}

// 187
enum BuiltInFunction {
    Abs, Acos, Asin, Atan, Blength, Cos, Exists, Exp, Format, Hibound, HiIndex,
    Length, LoBound, LoIndex, Log, Log2, Log10, Nvl, Odd, RolesOf, Sin, SizeOf,
    Sqrt, Tan, Typeof, Usedin, Value, ValueIn, ValueUnique
}
fn built_in_function(s: &str) -> IResult<BuiltInFunction> {
    use BuiltInFunction::*;
    // Tokenize then match the keyword, instead of doing a huge alt(...)
    let (rest, kw) = alpha1(s)?;
    Ok((rest, match kw {
        "abs" => Abs,
        "acos" => Acos,
        "asin" => Asin,
        "atan" => Atan,
        "blength" => Blength,
        "cos" => Cos,
        "exists" => Exists,
        "exp" => Exp,
        "format" => Format,
        "hibound" => Hibound,
        "hiindex" => HiIndex,

        "length" => Length,
        "lobound" => LoBound,
        "loindex" => LoIndex,
        "log" => Log,
        "log2" => Log2,
        "log10" => Log10,
        "nvl" => Nvl,
        "odd" => Odd,
        "rolesof" => RolesOf,
        "sin" => Sin,
        "sizeof" => SizeOf,

        "sqrt" => Sqrt,
        "tan" => Tan,
        "typeof" => Typeof,
        "usedin" => Usedin,
        "value" => Value,
        "value_in" => ValueIn,
        "value_unique" => ValueUnique,
        _ => return build_err(s, "No such built-in function"),
    }))
}

// 193
enum ConcreteTypes<'a> {
    Aggregation(AggregationTypes<'a>),
    Simple(SimpleTypes<'a>),
    TypeRef(TypeRef<'a>),
}
fn concrete_types(s: &str) -> IResult<ConcreteTypes> {
    use ConcreteTypes::*;
    alt((
        map(aggregation_types, Aggregation),
        map(simple_types, Simple),
        map(type_ref, TypeRef),
    ))(s)
}

// 196
enum ConstantFactor<'a> {
    BuiltIn(BuiltInConstant),
    ConstantRef(ConstantRef<'a>),
}
fn constant_factor(s: &str) -> IResult<ConstantFactor> {
    use ConstantFactor::*;
    alt((
        map(built_in_constant, BuiltIn),
        map(constant_ref, ConstantRef),
    ))(s)
}

// 197
struct ConstantId<'a>(SimpleId<'a>);
fn constant_id(s: &str) -> IResult<ConstantId> {
    map(simple_id, ConstantId)(s)
}

// 198
enum ConstructedTypes<'a> {
    Enumeration(EnumerationType<'a>),
    Select(SelectType<'a>),
}
fn constructed_types(s: &str) -> IResult<ConstructedTypes> {
    use ConstructedTypes::*;
    alt((
        map(enumeration_type, Enumeration),
        map(select_type, Select),
    ))(s)
}

// 202
struct DomainRule<'a> {
    rule_label_id: Option<RuleLabelId<'a>>,
    expression: Expression<'a>,
}
fn domain_rule(s: &str) -> IResult<DomainRule> {
    map(pair(opt(terminated(ws(rule_label_id), ws(char(':')))), expression),
         |(rule_label_id, expression)| DomainRule {rule_label_id, expression})
        (s)
}

// 203
struct Element<'a>(Expression<'a>, Option<Repetition<'a>>);
fn element(s: &str) -> IResult<Element> {
    map(pair(ws(expression), opt(preceded(ws(char(':')), repetition))),
        |(a, b)| Element(a, b))(s)
}

// 205
struct EntityConstructor<'a> {
    entity_ref: EntityRef<'a>,
    args: Vec<Expression<'a>>,
}
fn entity_constructor(s: &str) -> IResult<EntityConstructor> {
    map(pair(
        ws(entity_ref),
        delimited(
            ws(char('(')),
            separated_list0(ws(char(',')), ws(expression)),
            ws(char(')')),
        )), |(r, a)| EntityConstructor { entity_ref: r, args: a} )(s)
}

// 208
struct EntityId<'a>(SimpleId<'a>);
fn entity_id(s: &str) -> IResult<EntityId> {
    map(simple_id, EntityId)(s)
}

// 209
struct EnumerationExtension<'a> {
    type_ref: TypeRef<'a>,
    enumeration_items: Option<EnumerationItems<'a>>,
}
fn enumeration_extension(s: &str) -> IResult<EnumerationExtension> {
    map(preceded(
        ws(tag("based_on")),
        pair(type_ref, opt(preceded(ws(tag("with")), enumeration_items)))),
        |(a, b)| EnumerationExtension { type_ref: a, enumeration_items: b })(s)
}

// 210
struct EnumerationId<'a>(SimpleId<'a>);
fn enumeration_id(s: &str) -> IResult<EnumerationId> {
    map(simple_id, EnumerationId)(s)
}

// 211
struct EnumerationItems<'a>(Vec<EnumerationId<'a>>);
fn enumeration_items(s: &str) -> IResult<EnumerationItems> {
    map(delimited(
        ws(char('(')),
        separated_list1(ws(char(',')), ws(enumeration_id)),
        ws(char(')'))), EnumerationItems)(s)
}

// 212
struct EnumerationReference<'a>(Option<TypeRef<'a>>, EnumerationRef<'a>);
fn enumeration_reference(s: &str) -> IResult<EnumerationReference> {
    map(tuple((
        ws(opt(terminated(ws(type_ref), char('.')))),
        enumeration_ref
    )), |(a, b)| EnumerationReference(a, b))(s)
}

// 213
enum EnumerationItemsOrExtension<'a> {
    Items(EnumerationItems<'a>),
    Extension(EnumerationExtension<'a>),
}
struct EnumerationType<'a> {
    extensible: bool,
    items_or_extension: Option<EnumerationItemsOrExtension<'a>>
}
fn enumeration_type(s: &str) -> IResult<EnumerationType> {
    map(tuple((
        ws(opt(tag("extensible"))),
        ws(tag("enumeration")),
        ws(opt(alt((
            map(preceded(ws(tag("of")), enumeration_items),
                EnumerationItemsOrExtension::Items),
            map(enumeration_extension,
                EnumerationItemsOrExtension::Extension)))))
    )), |(e, _, p)| EnumerationType {
        extensible: e.is_some(),
        items_or_extension: p })(s)
}

// 216
struct Expression<'a>(SimpleExpression<'a>, Option<(RelOpExtended, SimpleExpression<'a>)>);
fn expression(s: &str) -> IResult<Expression> {
    map(pair(simple_expression, opt(pair(rel_op_extended, simple_expression))),
        |(a, b)| Expression(a, b))(s)
}

// 217
struct Factor<'a>(SimpleFactor<'a>, Option<SimpleFactor<'a>>);
fn factor(s: &str) -> IResult<Factor> {
    map(pair(simple_factor, opt(preceded(tag("**"), simple_factor))),
        |(a, b)| Factor(a, b))(s)
}

// 219
enum BuiltInOrFunctionRef<'a> {
    BuiltIn(BuiltInFunction),
    Ref(FunctionRef<'a>),
}
struct FunctionCall<'a>(BuiltInOrFunctionRef<'a>, ActualParameterList<'a>);
fn function_call(s: &str) -> IResult<FunctionCall> {
    map(pair(
            ws(alt((map(built_in_function, BuiltInOrFunctionRef::BuiltIn),
                    map(function_ref, BuiltInOrFunctionRef::Ref)))),
            actual_parameter_list),
        |(a, b)| FunctionCall(a, b))(s)
}

// 222
struct FunctionId<'a>(SimpleId<'a>);
fn function_id(s: &str) -> IResult<FunctionId> {
    map(simple_id, FunctionId)(s)
}

// 228
enum GeneralRef<'a> {
    Parameter(ParameterRef<'a>),
    Variable(VariableRef<'a>),
    _SimpleId(SimpleId<'a>),
}
fn general_ref(s: &str) -> IResult<GeneralRef> {
    map(simple_id, GeneralRef::_SimpleId)(s)
}


// 232
struct GroupQualifier<'a>(EntityRef<'a>);
fn group_qualifier(s: &str) -> IResult<GroupQualifier> {
    map(preceded(ws(char('\\')), entity_ref), GroupQualifier)(s)
}

// 236
struct Index<'a>(NumericalExpression<'a>);
fn index(s: &str) -> IResult<Index> {
    map(numerical_expression, Index)(s)
}

// 237
struct Index1<'a>(Index<'a>);
fn index_1(s: &str) -> IResult<Index1> {
    map(index, Index1)(s)
}

// 238
struct Index2<'a>(Index<'a>);
fn index_2(s: &str) -> IResult<Index2> {
    map(index, Index2)(s)
}

// 239
struct IndexQualifier<'a>(Index1<'a>, Index2<'a>);
fn index_qualifier(s: &str) -> IResult<IndexQualifier> {
    map(tuple((
        ws(char('[')),
        ws(index_1),
        ws(char(';')),
        ws(index_2),
        ws(char(']')),
    )), |(_, a, _, b, _)| IndexQualifier(a, b))(s)
}

// 240
enum InstantiableType<'a> {
    Concrete(ConcreteTypes<'a>),
    EntityRef(EntityRef<'a>),
}
fn instantiable_type(s: &str) -> IResult<InstantiableType> {
    use InstantiableType::*;
    alt((
        map(concrete_types, Concrete),
        map(entity_ref, EntityRef),
    ))(s)
}

// 243
struct Interval<'a> {
    low: IntervalLow<'a>,
    op1: IntervalOp,
    item: IntervalItem<'a>,
    op2: IntervalOp,
    high: IntervalHigh<'a>,
}
fn interval(s: &str) -> IResult<Interval> {
    map(delimited(
        ws(char('{')),
        ws(tuple((
            interval_low,
            interval_op,
            interval_item,
            interval_op,
            interval_high,
        ))),
        char('}')),
        |(low, op1, item, op2, high)| Interval { low, op1, item, op2, high })
    (s)
}

// 244
struct IntervalHigh<'a>(SimpleExpression<'a>);
fn interval_high(s: &str) -> IResult<IntervalHigh> {
    map(simple_expression, IntervalHigh)(s)
}

// 245
struct IntervalItem<'a>(SimpleExpression<'a>);
fn interval_item(s: &str) -> IResult<IntervalItem> {
    map(simple_expression, IntervalItem)(s)
}

// 246
struct IntervalLow<'a>(SimpleExpression<'a>);
fn interval_low(s: &str) -> IResult<IntervalLow> {
    map(simple_expression, IntervalLow)(s)
}

// 247
enum IntervalOp { LessThan, LessThanOrEqual }
fn interval_op(s: &str) -> IResult<IntervalOp> {
    alt((
        map(char('<'), |_| IntervalOp::LessThan),
        map(tag("<="), |_| IntervalOp::LessThanOrEqual),
    ))(s)
}

// 250
struct ListType<'a> {
    bounds: BoundSpec<'a>,
    unique: bool,
    instantiable_type: Box<InstantiableType<'a>>,
}
fn list_type(s: &str) -> IResult<ListType> {
    map(tuple((
        ws(tag("list")),
        ws(bound_spec),
        ws(tag("of")),
        ws(opt(tag("unique"))),
        ws(instantiable_type),
    )),
    |(_, b, _, uniq, t)| ListType {
        bounds: b,
        unique: uniq.is_some(),
        instantiable_type: Box::new(t),
    })(s)
}

// 251
enum Literal {
    String(String),
    Binary(usize),
    Logical(LogicalLiteral),
    Real(f64),
}
fn literal(s: &str) -> IResult<Literal> {
    use Literal::*;
    alt((
        map(binary_literal, Binary),
        map(string_literal, String),
        map(logical_literal, Logical),
        map(real_literal, Real)
    ))(s)
}

// 254
struct LogicalExpression<'a>(Expression<'a>);
fn logical_expression(s: &str) -> IResult<LogicalExpression> {
    map(expression, LogicalExpression)(s)
}

// 255
enum LogicalLiteral {
    True, False, Unknown
}
fn logical_literal(s: &str) -> IResult<LogicalLiteral> {
    alt((map(tag("false"),   |_| LogicalLiteral::False),
         map(tag("true"),    |_| LogicalLiteral::True),
         map(tag("unknown"), |_| LogicalLiteral::Unknown)))(s)
}

// 257
enum MultiplicationLikeOp {Mul, Div, IntegerDiv, Mod, And, ComplexEntity }
fn multiplication_like_op(s: &str) -> IResult<MultiplicationLikeOp> {
    use MultiplicationLikeOp::*;
    alt((
        map(char('*'),  |_| Mul),
        map(char('/'),  |_| Div),
        map(tag("div"), |_| IntegerDiv),
        map(tag("mod"), |_| Mod),
        map(tag("||"),  |_| ComplexEntity),
    ))(s)
}

// 258
enum NamedTypes<'a> {
    Entity(EntityRef<'a>),
    Type(TypeRef<'a>),
    _EntityOrTypeRef(SimpleId<'a>), // Used before disambiguation
}
fn named_types(s: &str) -> IResult<NamedTypes> {
    map(simple_id, NamedTypes::_EntityOrTypeRef)(s)
}

// 262
struct NumericalExpression<'a>(SimpleExpression<'a>);
fn numerical_expression(s: &str) -> IResult<NumericalExpression> {
    map(simple_expression, NumericalExpression)(s)
}

// 264
struct Parameter<'a>(Expression<'a>);
fn parameter(s: &str) -> IResult<Parameter> {
    map(expression, Parameter)(s)
}

// 265
struct ParameterId<'a>(SimpleId<'a>);
fn parameter_id(s: &str) -> IResult<ParameterId> {
    map(simple_id, ParameterId)(s)
}

// 267
struct Population<'a>(EntityRef<'a>);
fn population(s: &str) -> IResult<Population> {
    map(entity_ref, Population)(s)
}

// 268
struct PrecisionSpec<'a>(NumericalExpression<'a>);
fn precision_spec(s: &str) -> IResult<PrecisionSpec> {
    map(numerical_expression, PrecisionSpec)(s)
}

// 269
enum Primary<'a> {
    Literal(Literal),
    Qualifiable(QualifiableFactor<'a>, Vec<Qualifier<'a>>),
}
fn primary(s: &str) -> IResult<Primary> {
    use Primary::*;
    alt((
        map(literal, Literal),
        map(pair(qualifiable_factor, many0(qualifier)),
            |(f, qs)| Qualifiable(f, qs))
    ))(s)
}

// 274
enum QualifiableFactor<'a> {
    AttributeRef(AttributeRef<'a>),
    ConstantFactor(ConstantFactor<'a>),
    FunctionCall(FunctionCall<'a>),
    GeneralRef(GeneralRef<'a>),
    Population(Population<'a>),

    // catch-all for attribute, constant, general, population
    _SimpleId(SimpleId<'a>),
}
fn qualifiable_factor(s: &str) -> IResult<QualifiableFactor> {
    use QualifiableFactor::*;
    alt((
        map(simple_id, _SimpleId),
        map(constant_factor, ConstantFactor),
        map(function_call, FunctionCall),
    ))(s)
}

// 276
enum Qualifier<'a> {
    Attribute(AttributeQualifier<'a>),
    Group(GroupQualifier<'a>),
    Index(IndexQualifier<'a>),
}
fn qualifier(s: &str) -> IResult<Qualifier> {
    use Qualifier::*;
    alt((
        map(attribute_qualifier, Attribute),
        map(group_qualifier, Group),
        map(index_qualifier, Index),
    ))(s)
}

// 277
struct QueryExpression<'a> {
    var: VariableId<'a>,
    aggregate: AggregateSource<'a>,
    logical_expression: LogicalExpression<'a>,
}
fn query_expression(s: &str) -> IResult<QueryExpression> {
    map(tuple((
        ws(tag("QUERY")),
        ws(char('(')),
        ws(variable_id),
        ws(tag("<*")),
        ws(aggregate_source),
        ws(char('|')),
        ws(logical_expression,),
        ws(char(')')),
    )), |(_, _, var, _, aggregate, _, log, _)| QueryExpression {
        var,
        aggregate,
        logical_expression: log,
    })(s)
}

// 282
enum RelOp { LessThan, GreaterThan, LessThanOrEqual, GreaterThanOrEqual,
             NotEqual, Equal, InstanceEqual, InstanceNotEqual }
fn rel_op(s: &str) -> IResult<RelOp> {
    use RelOp::*;
    alt((
        map(char('<'),   |_| LessThan),
        map(char('>'),   |_| GreaterThan),
        map(tag("<="),   |_| LessThanOrEqual),
        map(tag(">="),   |_| GreaterThanOrEqual),
        map(tag("<>"),   |_| NotEqual),
        map(char('='),   |_| Equal),
        map(tag(":<>:"), |_| InstanceEqual),
        map(tag(":=:"),  |_| InstanceNotEqual),
    ))(s)
}

// 283
enum RelOpExtended { RelOp(RelOp), In, Like }
fn rel_op_extended(s: &str) -> IResult<RelOpExtended> {
    use RelOpExtended::*;
    alt((
        map(tag("in"),   |_| In),
        map(tag("like"), |_| Like),
        map(rel_op, RelOp)))(s)
}

// 287
struct Repetition<'a>(NumericalExpression<'a>);
fn repetition(s: &str) -> IResult<Repetition> {
    map(numerical_expression, Repetition)(s)
}

// 294
struct RuleLabelId<'a>(SimpleId<'a>);
fn rule_label_id(s: &str) -> IResult<RuleLabelId> {
    map(simple_id, RuleLabelId)(s)
}

// 300
struct SelectExtension<'a> {
    type_ref: TypeRef<'a>,
    select_list: Option<SelectList<'a>>,
}
fn select_extension(s: &str) -> IResult<SelectExtension> {
    map(tuple((
        ws(tag("based_on")), type_ref,
        opt(preceded(ws(tag("with")), select_list))
    )), |(_, a, b)| SelectExtension {
        type_ref: a, select_list: b
    })(s)
}

// 301
struct SelectList<'a>(Vec<NamedTypes<'a>>);
fn select_list(s: &str) -> IResult<SelectList> {
    map(delimited(
        ws(char('(')),
        separated_list1(ws(char(',')), ws(named_types)),
        char(')')),
        SelectList)(s)
}

// 302
enum SelectListOrExtension<'a> {
    List(SelectList<'a>),
    Extension(SelectExtension<'a>),
}
struct SelectType<'a> {
    extensible: bool,
    generic_entity: bool,
    list_or_extension: SelectListOrExtension<'a>,
}
fn select_type(s: &str) -> IResult<SelectType> {
    map(tuple((
        opt(pair(ws(tag("extensible")), opt(ws(tag("generic_entity"))))),
        ws(tag("select")),
        alt((
            map(select_list, SelectListOrExtension::List),
            map(select_extension, SelectListOrExtension::Extension),
        ))
    )), |(a, _, c)| SelectType{
        extensible: a.is_some(),
        generic_entity: a.is_some() && a.unwrap().1.is_some(),
        list_or_extension: c
    })(s)
}

// 303
struct SetType<'a> {
    bounds: BoundSpec<'a>,
    instantiable_type: Box<InstantiableType<'a>>,
}
fn set_type(s: &str) -> IResult<SetType> {
    map(tuple((
        ws(tag("set")),
        ws(bound_spec),
        ws(tag("of")),
        ws(instantiable_type),
    )),
    |(_, b, _, t)| SetType {
        bounds: b,
        instantiable_type: Box::new(t),
    })(s)
}

// 305
struct SimpleExpression<'a>(Box<Term<'a>>, Option<(AddLikeOp, Box<Term<'a>>)>);
fn simple_expression(s: &str) -> IResult<SimpleExpression> {
    map(pair(term, opt(pair(add_like_op, term))),
        |(a, b)| SimpleExpression(Box::new(a),
                                  b.map(|p| (p.0, Box::new(p.1)))))(s)
}

// 304
fn sign(s: &str) -> IResult<char> {
    alt((char('+'), char('-')))(s)
}

// 305
struct Term<'a>(Factor<'a>, Option<(MultiplicationLikeOp, Factor<'a>)>);
fn term(s: &str) -> IResult<Term> {
    map(pair(factor, opt(pair(multiplication_like_op, factor))),
        |(a, b)| Term(a, b))(s)
}

// 306
enum ExpressionOrPrimary<'a> {
    Expression(Box<Expression<'a>>),
    Primary(Primary<'a>),
}
enum SimpleFactor<'a> {
    AggregateInitializer(AggregateInitializer<'a>),
    EntityConstructor(EntityConstructor<'a>),
    EnumerationReference(EnumerationReference<'a>),
    Interval(Interval<'a>),
    QueryExpression(QueryExpression<'a>),
    Unary(Option<UnaryOp>, ExpressionOrPrimary<'a>)
}
fn simple_factor(s: &str) -> IResult<SimpleFactor> {
    use SimpleFactor::*;
    alt((
        map(aggregate_initializer, AggregateInitializer),
        map(entity_constructor, EntityConstructor),
        map(enumeration_reference, EnumerationReference),
        map(interval, Interval),
        map(query_expression, QueryExpression),
        map(pair(opt(ws(unary_op)), alt((
            map(delimited(ws(char('(')), ws(expression), char(')')),
                |e| ExpressionOrPrimary::Expression(Box::new(e))),
            map(primary, ExpressionOrPrimary::Primary)))),
            |(op, p)| Unary(op, p))
    ))(s)
}

// 307
enum SimpleTypes<'a> {
    Binary(Option<WidthSpec<'a>>), Boolean, Integer, Logical, Number,
    Real(Option<PrecisionSpec<'a>>), String(Option<WidthSpec<'a>>),
}
fn simple_types(s: &str) -> IResult<SimpleTypes> {
    use SimpleTypes::*;
    alt((
        map(preceded(ws(tag("binary")), opt(width_spec)), Binary),
        map(tag("boolean"), |_| Boolean),
        map(tag("integer"), |_| Integer),
        map(tag("logical"), |_| Logical),
        map(tag("number"), |_| Number),
        map(preceded(ws(tag("real")), opt(
            delimited(
                ws(char('(')),
                ws(precision_spec),
                char(')')),
            )), Real),
        map(preceded(ws(tag("string")), opt(width_spec)), String),
    ))(s)
}

// 310
fn string_literal(s: &str) -> IResult<String> {
    alt((simple_string_literal, encoded_string_literal))(s)
}

// 327
struct TypeDecl<'a> {
    type_id: TypeId<'a>,
    underlying_type: UnderlyingType<'a>,
    where_clause: Option<WhereClause<'a>>,
}
fn type_decl(s: &str) -> IResult<TypeDecl> {
    map(tuple((
        ws(tag("type")),
        ws(type_id),
        ws(char('=')),
        ws(underlying_type),
        ws(char(';')),
        ws(opt(where_clause)),
        ws(tag("end_type")),
        ws(char(';')),
    )), |(_, t, _, u, _, w, _, _)| TypeDecl {
        type_id: t,
        underlying_type: u,
        where_clause: w,
    })(s)
}

// 328
struct TypeId<'a>(SimpleId<'a>);
fn type_id(s: &str) -> IResult<TypeId> {
    map(simple_id, TypeId)(s)
}

// 331
enum UnaryOp { Add, Sub, Not }
fn unary_op(s: &str) -> IResult<UnaryOp> {
    use UnaryOp::*;
    alt((
        map(char('+'),  |_| Add),
        map(char('-'),  |_| Sub),
        map(tag("not"), |_| Not),
    ))(s)
}

// 332
enum UnderlyingType<'a> {
    Concrete(ConcreteTypes<'a>),
    Constructed(ConstructedTypes<'a>),
}
fn underlying_type(s: &str) -> IResult<UnderlyingType> {
    use UnderlyingType::*;
    alt((
        map(concrete_types, Concrete),
        map(constructed_types, Constructed),
    ))(s)
}

// 338
struct WhereClause<'a>(Vec<DomainRule<'a>>);
fn where_clause(s: &str) -> IResult<WhereClause> {
    map(preceded(
            ws(tag("where")),
            many1(terminated(ws(domain_rule), ws(char(';'))))),
        |v| WhereClause(v))(s)
}

// 337
struct VariableId<'a>(SimpleId<'a>);
fn variable_id(s: &str) -> IResult<VariableId> {
    map(simple_id, VariableId)(s)
}

// 340
struct Width<'a>(NumericalExpression<'a>);
fn width(s: &str) -> IResult<Width> {
    map(numerical_expression, Width)(s)
}

// 341
struct WidthSpec<'a> { expression: Width<'a>, fixed: bool }
fn width_spec(s: &str) -> IResult<WidthSpec> {
    map(tuple((
        ws(char('(')),
        ws(width),
        ws(char(')')),
        opt(tag("fixed"))
    )), |(_, w, _, f)| WidthSpec { expression: w, fixed: f.is_some() })(s)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_real_literal() {
        assert!(real_literal("1.E6").unwrap().1 == 1.0e6);
        assert!(real_literal("3.5e-5").unwrap().1 == 3.5e-5);
        assert!(real_literal("359.62").unwrap().1 == 359.62);
    }
    #[test]
    fn test_octet() {
        assert_eq!(octet("00").unwrap().1, "00");
    }
    #[test]
    fn test_encoded_character() {
        assert_eq!(encoded_character("00000041").unwrap().1, 'A');
    }
    #[test]
    fn test_encoded_string_literal() {
        assert_eq!(&encoded_string_literal("\"\"").unwrap().1, "");
        assert_eq!(&encoded_string_literal("\"00000041\"").unwrap().1, "A");
        assert_eq!(&encoded_string_literal("\"0000795e00006238\"").unwrap().1, "神戸");
    }
    #[test]
    fn test_simple_string_literal() {
        assert_eq!(simple_string_literal("'omg'").unwrap().1, "omg");
        assert_eq!(simple_string_literal("'om''g'").unwrap().1, "om'g");
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Remove comments from an EXPRESS file and converts to lower-case
pub fn strip_flatten(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        match data[i] {
            // Block comments
            b'(' => if i + 1 < data.len() && data[i + 1] == b'*' {
                for j in memchr_iter(b')', &data[i + 2..]) {
                    if data[i + j + 1] == b'*' {
                        i += j + 2;
                        break;
                    }
                }
            },
            // Single-line comments
            b'-' if i + 1 < data.len() && data[i + 1] == b'-' => {
                let newline = memchr(b'\n', &data[i + 2..]);
                i += newline.unwrap_or(0) + 3;
            },
            c => out.push(c.to_ascii_lowercase())
        }
        i += 1;
    }
    out
}

