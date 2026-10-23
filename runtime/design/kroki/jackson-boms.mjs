import { readFileSync, writeFileSync } from 'node:fs';
const path = '/source/server/pom.xml';
const source = readFileSync(path, 'utf8');
if (source.includes('<dependencyManagement>') || !source.includes('  <dependencies>')) throw new Error('upstream POM structure changed');
const boms = `<dependencyManagement><dependencies>
<dependency><groupId>com.fasterxml.jackson</groupId><artifactId>jackson-bom</artifactId><version>2.22.3</version><type>pom</type><scope>import</scope></dependency>
<dependency><groupId>tools.jackson</groupId><artifactId>jackson-bom</artifactId><version>3.1.7</version><type>pom</type><scope>import</scope></dependency>
</dependencies></dependencyManagement>`;
writeFileSync(path, source.replace('  <dependencies>', `${boms}\n  <dependencies>`));
