import hashlib
import time
import random
import struct

def aequihash_light(header, nonce, difficulty):
    """Simulation simplifiée de l'algorithme AequiHash"""
    data = header + struct.pack("<Q", nonce)
    # On simule le coût mémoire par plusieurs passages de SHA3
    h = hashlib.sha3_256(data).digest()
    for _ in range(10): # Simulation du mixage mémoire-hard
        h = hashlib.sha3_256(h + data).digest()
    return h

def main():
    print("========================================")
    print("   AEQUITAS MINING SIMULATOR (v0.1.0)")
    print("========================================")
    print("Algorithme: AequiHash (GPU optimized)")
    print("VRAM Min: 6 GB (Simulation)")
    print("----------------------------------------")
    
    address = "aeq1qrp3uqu76r8re3u4sh5re6sh8sh8sh1"
    target_difficulty = 1000000 
    target = (1 << 256) // target_difficulty
    
    header = b"Aequitas_Genesis_Block_Header_Simulation"
    nonce = random.randint(0, 1000000)
    hashes = 0
    start_time = time.time()
    
    print(f"Mining with address: {address}")
    print(f"Target Difficulty: {target_difficulty}")
    print("Mining started... Press Ctrl+C to stop.\n")
    
    try:
        while True:
            hash_result = aequihash_light(header, nonce, target_difficulty)
            hash_int = int.from_bytes(hash_result, 'big')
            
            hashes += 1
            nonce += 1
            
            # Affichage des stats toutes les 2 secondes
            if hashes % 50 == 0:
                elapsed = time.time() - start_time
                hashrate = hashes / elapsed
                print(f"\rHashrate: {hashrate:.2f} H/s | Hashes: {hashes} | Nonce: {nonce}", end="")
            
            # Vérification de la solution
            if hash_int <= target:
                print(f"\n\n[SOLVED] SOLUTION FOUND!")
                print(f"Nonce: {nonce}")
                print(f"Hash:  {hash_result.hex()}")
                print(f"Result: {hash_int}")
                print("----------------------------------------")
                print("Submission to node: SUCCESS!")
                break
                
    except KeyboardInterrupt:
        print("\n\nMining stopped by user.")

if __name__ == "__main__":
    main()
