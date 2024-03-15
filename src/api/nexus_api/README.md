# About this directory
The structs here model the JSON responses by the Nexusmods API (with the exception of "FileList" being "Files".
Responses are deserialized by serde, which requires the struct field names to match those in the JSON responses.
Struct names can be arbitrary, but have been chosen to match the API requests which return them.
