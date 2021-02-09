# modified from
# https://github.com/awslabs/amazon-eks-ami/pull/494#issuecomment-648247660

import requests
from bs4 import BeautifulSoup

response = requests.get("https://docs.aws.amazon.com/AWSEC2/latest"
                        "/UserGuide/using-eni.html#AvailableIpPerENI")

parsed_html = BeautifulSoup(response.text, features="html.parser")

tables = parsed_html.findAll('table')
for table in tables:
    if len(table) > 50:
        t = table

rows = t.find_all("tr")

for row in rows:
    cells = row.find_all("td")
    if len(cells) < 1:
        continue

    try:
        instance_type = cells[0].text.strip()
        instance_enis = int(cells[1].text.strip())
        ips_per_eni = int(cells[2].text.strip())
    except ValueError as e:
        continue

    max_pods = (instance_enis - 1) * (ips_per_eni -1) + 2
    print(f'{instance_type} {max_pods}')
