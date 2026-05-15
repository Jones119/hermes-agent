
#[cfg(test)]
mod tests {
    use hermes_core::tool::Tool;
    use hermes_tools::{ReadFileTool, WriteFileTool};

    #[tokio::test]
    async fn test_read_file_tool() {
        let tool = ReadFileTool::new();
        assert_eq!(tool.name(), "read_file");
        assert_eq!(tool.description(), "Read the contents of a file");
    }

    #[tokio::test]
    async fn test_write_file_tool() {
        let tool = WriteFileTool::new();
        assert_eq!(tool.name(), "write_file");
        assert_eq!(tool.description(), "Write content to a file");
    }

    #[tokio::test]
    async fn test_read_file_parameters() {
        let tool = ReadFileTool::new();
        let params = tool.parameters();
        
        assert!(params.get("properties").is_some());
        let properties = params.get("properties").unwrap();
        assert!(properties.get("path").is_some());
        assert!(properties.get("offset").is_some());
        assert!(properties.get("limit").is_some());
    }

    #[tokio::test]
    async fn test_write_file_parameters() {
        let tool = WriteFileTool::new();
        let params = tool.parameters();
        
        assert!(params.get("properties").is_some());
        let properties = params.get("properties").unwrap();
        assert!(properties.get("path").is_some());
        assert!(properties.get("content").is_some());
        assert!(properties.get("append").is_some());
    }
}
