import pandas as pd
import sys
import os

csv_path = 'raw_measurements.csv'
if not os.path.exists(csv_path):
    print(f"File not found in current dir: {os.getcwd()}/{csv_path}")
    # Try looking one level up just in case
    if os.path.exists(os.path.join('qr_benchmark', csv_path)):
        csv_path = os.path.join('qr_benchmark', csv_path)

print(f"Reading from: {csv_path}")

try:
    df = pd.read_csv(csv_path)
    print(f"Total rows: {len(df)}")
    print("\nCategories found:", df['category'].unique())
    
    print("\nCounts per category:")
    print(df['category'].value_counts())
    
    # Check if we have successes in other categories
    print("\nSuccesses by category (Correct status count):")
    successes = df[df['status'] == 'Correct'].groupby('category').size()
    print(successes)

    print("\nLibraries found:", df['library'].unique())

except Exception as e:
    print(e)
