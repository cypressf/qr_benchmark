import pandas as pd

df = pd.read_csv('raw_measurements.csv')
non_decoding = df[df['category'] != 'decoding']

print("Status counts for non-decoding categories:")
print(non_decoding['status'].value_counts())

print("\nSample rows from non-decoding:")
print(non_decoding[['category', 'status', 'expected_text', 'decoded_text']].head(10))

