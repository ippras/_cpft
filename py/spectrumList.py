import os
import xml.etree.ElementTree as ET
import tkinter as tk
from tkinter import filedialog

def process_xml_file(filepath):
    """
    Parses an XML file, finds the <spectrumList> tag, and prints the filename
    and the value of its 'count' attribute, handling XML namespaces.
    """
    try:
        # Parse the XML file
        tree = ET.parse(filepath)
        root = tree.getroot()

        # Define the namespace dictionary
        # The default namespace in your XML is "http://psi.hupo.org/ms/mzml"
        # We'll map it to a prefix, e.g., 'mzml'
        namespaces = {
            'mzml': 'http://psi.hupo.org/ms/mzml',
            'xsi': 'http://www.w3.org/2001/XMLSchema-instance' # Though not strictly needed for spectrumList
        }

        # Find the <spectrumList> element using the namespace prefix
        # The format is '{namespace_uri}tag_name' or 'prefix:tag_name' with namespaces map
        # Using .// to search anywhere in the tree
        spectrum_list_element = root.find('.//mzml:spectrumList', namespaces)

        if spectrum_list_element is not None:
            count_value = spectrum_list_element.get('count')
            if count_value is not None:
                print(f"File: {os.path.basename(filepath)}, Count: {count_value}")
            else:
                print(f"File: {os.path.basename(filepath)}, <spectrumList> found but 'count' attribute is missing.")
        else:
            print(f"File: {os.path.basename(filepath)}, <spectrumList> tag not found.")
    except ET.ParseError as e:
        print(f"Error parsing XML file {os.path.basename(filepath)}: {e}")
    except Exception as e:
        print(f"An unexpected error occurred with file {os.path.basename(filepath)}: {e}")

def find_spectrum_counts_in_directory(directory_path):
    """
    Iterates through all files in the specified directory, processes XML files
    to find <spectrumList count="..."> and prints the results.
    """
    if not directory_path: # If user cancels the dialog, directory_path will be an empty string
        print("Directory selection cancelled.")
        return

    if not os.path.isdir(directory_path):
        print(f"Error: Directory '{directory_path}' not found or is not a valid directory.")
        return

    print(f"Searching for <spectrumList count> in XML files in: {directory_path}\n")

    found_xml_files = False
    for filename in os.listdir(directory_path):
        if filename.endswith(".mzML"):
            found_xml_files = True
            filepath = os.path.join(directory_path, filename)
            process_xml_file(filepath)
    
    if not found_xml_files:
        print(f"No XML files found in the directory: {directory_path}")

if __name__ == "__main__":
    # Create a Tkinter root window, but hide it
    root = tk.Tk()
    root.withdraw() # Hide the main window

    # Open a directory selection dialog
    selected_directory = filedialog.askdirectory(
        title="Select Directory Containing XML Files"
    )

    # Pass the selected directory to the processing function
    find_spectrum_counts_in_directory(selected_directory)