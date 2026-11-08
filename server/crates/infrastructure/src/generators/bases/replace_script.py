import base64

search_text = base64.b64decode(b"{}").decode('utf-8')
replace_text = base64.b64decode(b"{}").decode('utf-8')

with open("{}", 'r') as file:
    data = file.read()
    data = data.replace(search_text, replace_text)

with open("{}", 'w') as file:
    file.write(data)

