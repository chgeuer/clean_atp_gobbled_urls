# Clean up gobbled links in my clipboard

This utility watches my Windows clipboard, and when it finds a redirect URL to the Advanced Thread Protection, which obfuscates the link target, it strips the redirect part away, and replaces it with the 'clean' URL. 

## Why

I take many notes and minutes, and share them with customers. Sometimes I copy/paste links from e-mails. I put links into source code. I put links into documentation. I put links into blogs and chat. 

> I want my links to look clean. Period.

However, my employer (Microsoft) has the desire to protect me from phishing attacks, which happen when I click on shady links. So we have this advanced threat protection system, which replaces all links in incoming email, sometimes also in Teams, with things which look like this:


```
https://nam06.safelinks.protection.outlook.com/?url=https%3A%2F%2Fgithub.com%2FAzure%2Fazure-storage-azcopy%2Freleases%2Ftag%2Fv10.9.0&data=04%7C01%7Cjohn.doe%40microsoft.com%7C9b0864bff05e4913db7e08d8dac206f3%7C72f988bf86f141af91ab2d7cd011db47%7C1%7C0%7C637499874161053585%7CUnknown%7CTWFpbGZsb3d8eyJWIjoiMC4wLjAwMDAiLCJQIjoiV2luMzIiLCJBTiI6Ik1haWwiLCJXVCI6Mn0%3D%7C1000&sdata=YbssDXAY%2FiypxjYrJONMO09VEyMI4j4VyCIUs098Lyk%3D&reserved=0
```

This link refers to `....safelinks.protection.outlook.com`, to dynamically check if the referenced URL is dangerous or not. And, this link contains PII, specifically this `john.doe%40microsoft.com` thing, the name of the employee with whom the link was shared. I don't want all that crap, I don't want to open the `...outlook.com` page in a browser, just to get redirected to the real thing, and then select the address bar, select everything, and copy the clean URL to my clipboard. This breaks my flow and is super annoying. 

Therefore, I keep this little helper sanitizing my clipboard for me.

> USE AT YOUR OWN RISK!!

```shell
grep --recursive --include '*.md' --files-with-matches nam06.safelinks.protection.outlook.com 
```

