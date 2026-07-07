import sys
print("Usage: python3 get_size.py [folder]. Gives the exact size of the folder in bytes.")

if sys.argv[1:]:
    import os
    folder = sys.argv[1]
    total_size = 0
    for root, dirnames, filenames in os.walk(folder):
        for f in filenames:
            fp = os.path.join(root, f)
            total_size += os.path.getsize(fp)
    print(f"Total size of {folder} is: {total_size/1024/1024:.2f} MB and {total_size} bytes.")
