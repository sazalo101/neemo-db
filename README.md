# neemo-db
# **Neemo** 🗄️
A lightweight, fast, and embedded **document database** powered by **Sled**.  
Neemo provides an easy-to-use **key-value** storage system for developers who need **persistent storage** without the complexity of a full database.

## 🚀 **Features**
✅ **Fast & Efficient**: Uses **Sled**, a high-performance embedded database.  
✅ **Simple API**: Store, retrieve, and delete JSON-like documents.  
✅ **Persistent Storage**: Data is stored on disk.  
✅ **Cross-platform**: Works on **Linux, macOS, and Windows**.  
✅ **Lightweight**: No external dependencies or extra database setup required.  

---

## 🛠 **Installation**
### **Install via Cargo**
```sh
cargo install neemo
```

### **Build from Source**
```sh
git clone https://github.com/yourusername/neemo.git
cd neemo
cargo build --release
```

This will generate an executable at:

```
target/release/neemo
```

Move it to `/usr/local/bin/` for global use:

```sh
cp target/release/neemo /usr/local/bin/neemo
```

---

## 🚀 **Usage**
### **Start the Neemo Server**
Run Neemo as a local database server:
```sh
neemo
```

By default, Neemo stores data in:
```
$HOME/.neemo/db
```

### **API Endpoints**
Neemo provides a **JSON API** for interacting with the database.

#### 📌 **Store Data (POST)**
```sh
curl -X POST http://localhost:5000/set -H "Content-Type: application/json" \
     -d '{"key": "user:123", "value": {"name": "Alice", "age": 25}}'
```

#### 📌 **Retrieve Data (GET)**
```sh
curl http://localhost:5000/get/user:123
```
**Response:**
```json
{"name": "Alice", "age": 25}
```

#### 📌 **Delete Data (DELETE)**
```sh
curl -X DELETE http://localhost:5000/delete/user:123
```

---

# 🔗 **Using Neemo with Flask (Python)**
You can interact with Neemo from **Flask** using `requests`:

### **Install Dependencies**
```sh
pip install flask requests
```

### **Flask Example**
```python
from flask import Flask, request, jsonify
import requests

app = Flask(__name__)

NEEMO_URL = "http://localhost:5000"

@app.route('/store', methods=['POST'])
def store():
    data = request.json
    res = requests.post(f"{NEEMO_URL}/set", json=data)
    return res.json()

@app.route('/get/<key>', methods=['GET'])
def get(key):
    res = requests.get(f"{NEEMO_URL}/get/{key}")
    return res.json()

@app.route('/delete/<key>', methods=['DELETE'])
def delete(key):
    res = requests.delete(f"{NEEMO_URL}/delete/{key}")
    return res.json()

if __name__ == "__main__":
    app.run(port=8080)
```

Now, you can store and retrieve data via:
```sh
curl -X POST http://localhost:8080/store -H "Content-Type: application/json" \
     -d '{"key": "product:101", "value": {"name": "Laptop", "price": 1200}}'

curl http://localhost:8080/get/product:101
```

---

# 🔗 **Using Neemo with Node.js**
You can interact with Neemo using `axios` in **Node.js**.

### **Install Dependencies**
```sh
npm install axios express
```

### **Node.js Example**
```javascript
const express = require("express");
const axios = require("axios");

const app = express();
const NEEMO_URL = "http://localhost:5000";

app.use(express.json());

app.post("/store", async (req, res) => {
    const response = await axios.post(`${NEEMO_URL}/set`, req.body);
    res.json(response.data);
});

app.get("/get/:key", async (req, res) => {
    const response = await axios.get(`${NEEMO_URL}/get/${req.params.key}`);
    res.json(response.data);
});

app.delete("/delete/:key", async (req, res) => {
    const response = await axios.delete(`${NEEMO_URL}/delete/${req.params.key}`);
    res.json(response.data);
});

app.listen(8080, () => console.log("Server running on port 8080"));
```

Now, you can interact with the database using:
```sh
curl -X POST http://localhost:8080/store -H "Content-Type: application/json" \
     -d '{"key": "user:456", "value": {"name": "Bob", "age": 30}}'

curl http://localhost:8080/get/user:456
```

---

## 📜 **License**
This project is licensed under the **MIT License**.

---

### **🎉 Now your database is ready to use in Python, Node.js, or any other language!** 🚀


