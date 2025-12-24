import pandas as pd

df = pd.read_csv('raw_measurements.csv')
incorrect = df[df['status'] == 'Incorrect']

print("Sample Incorrect comparisons:")
for index, row in incorrect.head(10).iterrows():
    print(f"Cat: {row['category']}")
    print(f"  Exp: '{row['expected_text']}'")
    print(f"  Got: '{row['decoded_text']}'")
    print("-" * 20)

