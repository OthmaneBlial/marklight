# Safe text

<script>alert("x")</script>

<iframe src="file:///etc/passwd"></iframe>

<img src="http://tracker" onerror="alert(1)">

<svg onload="alert(1)"></svg>

[bad](javascript:alert%281%29) [file](file:///etc/passwd) [data](data:text/html,x)

![bad](http://tracker/image.png)

<p onclick="evil()">Safe text</p>
