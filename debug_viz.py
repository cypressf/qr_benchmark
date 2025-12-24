import json
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from PIL import Image
import sys
import os

def visualize_debug(json_path="debug_output.json"):
    if not os.path.exists(json_path):
        print(f"Error: {json_path} not found.")
        return

    with open(json_path, 'r') as f:
        data = json.load(f)

    image_path = data['image_path']
    # Adjust path if needed
    if not os.path.exists(image_path):
        candidate = os.path.join("..", image_path)
        if os.path.exists(candidate):
            image_path = candidate
        else:
             print(f"Error: Image {image_path} not found.")
    
    try:
        img = Image.open(image_path)
    except Exception as e:
        print(f"Could not open image: {e}")
        return

    plt.figure(figsize=(12, 10))
    plt.imshow(img)
    ax = plt.gca()

    # Colors
    colors = {
        'GroundTruth': 'green',
        'rqrr': 'blue',
        'rxing': 'red',
        'bardecoder': 'orange'
    }

    # Plot Ground Truth
    if data.get('ground_truth_sets'):
        for set_idx, pts in enumerate(data['ground_truth_sets']):
            # Draw polygon
            poly = patches.Polygon(pts, linewidth=3, edgecolor=colors['GroundTruth'], facecolor='none', label='Ground Truth' if set_idx == 0 else "")
            ax.add_patch(poly)
            # Draw points
            for i, (x, y) in enumerate(pts):
                plt.plot(x, y, 'o', color=colors['GroundTruth'])
                # plt.text(x, y, str(i), color='white', fontsize=12, fontweight='bold')

    # Plot Detections
    for det in data['detections']:
        lib = det['library']
        status = det['status']
        points = det['points']
        
        print(f"Library: {lib}, Status: {status}")
        
        if points:
            color = colors.get(lib, 'purple')
            # Draw polygon
            poly = patches.Polygon(points, linewidth=2, edgecolor=color, facecolor='none', linestyle='--', label=f'{lib} ({status})')
            ax.add_patch(poly)
             # Draw points
            for i, (x, y) in enumerate(points):
                plt.plot(x, y, 'x', color=color)

    # Legend
    handles, labels = ax.get_legend_handles_labels()
    # Deduplicate legend labels
    by_label = dict(zip(labels, handles))
    plt.legend(by_label.values(), by_label.keys(), loc='upper right')
    
    plt.title(f"QR Detection Debug: {os.path.basename(image_path)}")
    plt.tight_layout()
    plt.savefig('debug_viz.png', dpi=150)
    print("Saved debug_viz.png")

if __name__ == "__main__":
    visualize_debug()
