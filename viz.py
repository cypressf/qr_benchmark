import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import os
import numpy as np

def generate_visualizations(csv_path="raw_measurements.csv"):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found.")
        return

    print("Loading data...")
    df = pd.read_csv(csv_path)

    # Preprocessing
    df['duration_ms'] = df['duration_us'] / 1000.0
    df['success_numeric'] = (df['status'] == 'Correct').astype(int)
    
    # Filter for correct decodes for performance metrics
    correct_decodes = df[df['status'] == 'Correct'].copy()

    sns.set_theme(style="whitegrid")
    
    # 1. Computation Time Distribution (KDE)
    print("Generating time distribution plot...")
    plt.figure(figsize=(12, 6))
    sns.kdeplot(data=correct_decodes, x='duration_ms', hue='library', fill=True, clip=(0, None))
    plt.title('Distribution of Decoding Times (Correct Decodes Only)', fontsize=16)
    plt.xlabel('Time (ms)')
    plt.xlim(0, correct_decodes['duration_ms'].quantile(0.95)) # Focus on the main distribution
    plt.tight_layout()
    plt.savefig('dist_time.png', dpi=300)
    plt.close()

    # 2. Success Rates by Defect Type with Error Bars
    print("Generating success rates by defect type plot...")
    plt.figure(figsize=(16, 8))
    # barplot automatically computes mean and 95% confidence interval for error bars
    ax = sns.barplot(
        data=df, 
        x='category', 
        y='success_numeric', 
        hue='library',
        errorbar=('ci', 95),
        capsize=.1
    )
    # Convert y-axis to percentage
    ax.set_yticklabels(['{:,.0%}'.format(x) for x in ax.get_yticks()])
    plt.title('Success Rates by Defect Type (with 95% CI)', fontsize=16)
    plt.ylabel('Success Rate')
    plt.xlabel('Defect Category')
    plt.xticks(rotation=45, ha='right')
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.tight_layout()
    plt.savefig('success_by_defect.png', dpi=300)
    plt.close()

    # 3. Performance by Defect Type (Mean with Error Bars)
    print("Generating mean performance by defect type plot...")
    plt.figure(figsize=(16, 8))
    # Point plot is good for comparing means across categories
    sns.pointplot(
        data=correct_decodes, 
        x='category', 
        y='duration_ms', 
        hue='library', 
        dodge=0.4, 
        join=False,
        errorbar=('ci', 95),
        capsize=.1
    )
    plt.title('Mean Decoding Time by Defect Type (Correct Decodes, 95% CI)', fontsize=16)
    plt.ylabel('Time (ms)')
    plt.xlabel('Defect Category')
    plt.xticks(rotation=45, ha='right')
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.tight_layout()
    plt.savefig('perf_by_defect_mean.png', dpi=300)
    plt.close()

    # 4. Performance Distribution by Defect Type (Box Plots)
    print("Generating performance distribution by defect type plot...")
    plt.figure(figsize=(16, 8))
    sns.boxplot(
        data=correct_decodes, 
        x='category', 
        y='duration_ms', 
        hue='library', 
        showfliers=False # Hide outliers to keep the scale readable
    )
    plt.title('Decoding Time Distribution by Defect Type (Correct Decodes, No Outliers)', fontsize=16)
    plt.ylabel('Time (ms)')
    plt.xlabel('Defect Category')
    plt.xticks(rotation=45, ha='right')
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.tight_layout()
    plt.savefig('perf_by_defect_dist.png', dpi=300)
    plt.close()
    
    print("Done generating plots.")

if __name__ == "__main__":
    generate_visualizations()
