#!/bin/bash


# Function to display usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -f FILE        Specify input file"
    echo "  -o OUTPUT      Specify output file"
    echo "  --backup       Create backup of files"
}

# Function to backup files
backup_files() {
    local dir=$1
    local backup_dir="backup_$(date +%Y%m%d_%H%M%S)"
    
    echo "Creating backup in $backup_dir..."
    mkdir -p "$backup_dir"
    
    # Copy files to backup directory
    find "$dir" -type f -name "*.txt" -exec cp {} "$backup_dir/" \;
    
    echo "Backup completed"
}

# Function to process files
process_files() {
    local input_file=$1
    local output_file=$2
    
    if [ ! -f "$input_file" ]; then
        echo "Error: Input file $input_file not found"
        exit 1
    }
    
    # Process the file
    echo "Processing $input_file..."
    while IFS= read -r line; do
        # Example processing: convert to uppercase
        echo "${line^^}" >> "$output_file"
    done < "$input_file"
    
    echo "Processing completed. Output saved to $output_file"
}

# Parse command line arguments
INPUT_FILE=""
OUTPUT_FILE="output.txt"
DO_BACKUP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -f)
            INPUT_FILE="$2"
            shift 2
            ;;
        -o)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --backup)
            DO_BACKUP=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Check if input file is provided
if [ -z "$INPUT_FILE" ]; then
    echo "Error: Input file not specified"
    show_usage
    exit 1
fi

# Main script execution
echo "Starting script execution..."

# Create backup if requested
if [ "$DO_BACKUP" = true ]; then
    backup_files "$(dirname "$INPUT_FILE")"
fi

# Process files
process_files "$INPUT_FILE" "$OUTPUT_FILE"

# Example of error handling and logging
if [ $? -eq 0 ]; then
    echo "Script completed successfully"
    # Log success
    logger -t "$(basename "$0")" "Processing completed successfully for $INPUT_FILE"
else
    echo "Script encountered errors"
    # Log error
    logger -t "$(basename "$0")" "Error processing $INPUT_FILE"
fi

# Cleanup function
cleanup() {
    # Add any cleanup tasks here
    echo "Performing cleanup..."
    # Example: Remove temporary files
    rm -f /tmp/temp_*.txt 2>/dev/null
}

# Register cleanup function to run on script exit
trap cleanup EXIT

exit 0