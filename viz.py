import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import os

def generate_visualizations(csv_path="raw_measurements.csv"):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found.")
        return

    print("Loading data...")
    df = pd.read_csv(csv_path)

    # Convert duration to milliseconds for easier reading
    df['duration_ms'] = df['duration_us'] / 1000.0

    # 1. Success Rates by Category and Library
    print("Generating success rate plot...")
    plt.figure(figsize=(15, 8))
    
    # Calculate success rates
    success_rates = df.groupby(['category', 'library'])['status'].apply(lambda x: (x == 'Correct').mean() * 100).reset_index()
    success_rates.rename(columns={'status': 'Success Rate (%)'}, inplace=True)
    
    sns.set_theme(style="whitegrid")
    ax = sns.barplot(data=success_rates, x='category', y='Success Rate (%)', hue='library')
    
    plt.title('QR Code Decoding Success Rate by Category', fontsize=16)
    plt.xticks(rotation=45, ha='right')
    plt.tight_layout()
    plt.savefig('success_rates_py.png', dpi=300)
    plt.close()

    # 2. Performance (Duration) Distribution for Correct Decodes
    print("Generating performance plot...")
    correct_decodes = df[df['status'] == 'Correct'].copy()
    
    if correct_decodes.empty:
        print("No correct decodes found, skipping performance plot.")
    else:
        plt.figure(figsize=(12, 6))
        
        # Use boxplot to show distribution
        # Log scale might be useful if variance is huge, but let's try linear first (or log y-axis)
        sns.boxplot(data=correct_decodes, x='library', y='duration_ms', showfliers=False) # Hide outliers for clarity
        
        plt.title('Decoding Duration Distribution (Correct Decodes Only)', fontsize=16)
        plt.ylabel('Duration (ms)')
        plt.tight_layout()
        plt.savefig('performance_py.png', dpi=300)
        plt.close()

        # 3. Overall Summary Table
        print("Generating summary table...")
        summary = df.groupby('library').agg(
            Total_Images=('file_path', 'nunique'),
            Total_Attempts=('status', 'count'),
            Success_Count=('status', lambda x: (x == 'Correct').sum()),
            Success_Rate=('status', lambda x: (x == 'Correct').mean() * 100),
            Median_Time_ms=('duration_ms', 'median'),
            Mean_Time_ms=('duration_ms', 'mean')
        ).round(2)
        
        # Calculate speed on correct only
        correct_stats = df[df['status'] == 'Correct'].groupby('library')['duration_ms'].agg(
            Median_Correct_ms='median',
            Mean_Correct_ms='mean'
        ).round(2)
        
        summary = summary.join(correct_stats)
        
        print("\nSummary Statistics:")
        print(summary)
        summary.to_csv('summary_stats.csv')

if __name__ == "__main__":
    generate_visualizations()

