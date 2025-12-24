import pandas as pd

df = pd.read_csv('raw_measurements.csv')
incorrect = df[df['status'] == 'Incorrect']

print("Decoded text counts for Incorrect status:")
print(incorrect['decoded_text'].value_counts())

