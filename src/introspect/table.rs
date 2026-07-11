/// Represents information about a database table, including its name and optional comment.   
/// [`DatabaseIntrospector::list_tables_with_info`](crate::introspect::DatabaseIntrospector::list_tables_with_info). 
/// The `comment` field is optional and may be `None` if no comment is associated with the table.
pub struct TableInfo {
    pub name: String,
    pub comment: Option<String>,
}