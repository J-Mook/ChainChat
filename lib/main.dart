import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:simple_icons/simple_icons.dart';
import './messages/generated.dart';
import './messages/chatmessage.pb.dart';

import 'package:sidebarx/sidebarx.dart';
import 'provider/InfoProvider.dart';
import 'provider/themeprovider.dart';
import 'package:provider/provider.dart';
import 'dart:io';


void main() async {
  await initializeRust();
  runApp(
    MultiProvider(
      providers: [
        ChangeNotifierProvider( create: (context) => ThemeProvider(), ),
        ChangeNotifierProvider( create: (context) => InfoProvider(), ),
      ],
      child: MainApp()
    )
  );
}

class MainApp extends StatelessWidget {
  const MainApp({super.key});

  @override
  Widget build(BuildContext context) {
    final themeProvider = Provider.of<ThemeProvider>(context);

    return MaterialApp(
      // home: Scaffold(
      //   body: Center(
      //     child: MainBody(),
      //   ),
      // ),
      title: "jmook",
      initialRoute: '/Enterance',
      routes: {
        '/Enterance' : (context) => EntrancePage(),
        '/MainPage' : (context) => MainBody(),
      },

      theme: ThemeData.light(),
      darkTheme: ThemeData.dark(),
      themeMode: themeProvider.themeMode,

    );
  }
}

class MainBody extends StatefulWidget {
  const MainBody({super.key});

  @override
  State<MainBody> createState() => _MainBody();
}

class _MainBody extends State<MainBody> {

  SidebarXController _controller = SidebarXController(selectedIndex: 0);

  @override
  Widget build(BuildContext context) {
    final themeProvider = Provider.of<ThemeProvider>(context);
    return Scaffold(
      body: Row(
        children: [
          Container(
            margin: EdgeInsets.fromLTRB(5, 5, 0, 5),
            child: SidebarX(
              controller: _controller,
              items: const [
                SidebarXItem(icon: Icons.chat, label: ' Chat'),
                SidebarXItem(icon: Icons.settings, label: ' Setting?'),
              ],
              theme: SidebarXTheme(
                width: 60,
                decoration: BoxDecoration(
                  color: Color.fromARGB(255, 76, 119, 140),
                  borderRadius: BorderRadius.circular(15),
                ),
                margin: EdgeInsets.only(right: 10),
              ),
              extendedTheme: SidebarXTheme(
                width: 150,
                decoration: BoxDecoration(
                  color: Color.fromARGB(255, 96, 125, 139),
                  borderRadius: BorderRadius.circular(15),
                ),
                margin: EdgeInsets.only(right: 20),
              ),
              footerBuilder: (context, extended) {
                return extended ?
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text("DarkMode"),
                    SizedBox(height: 35,
                      child: FittedBox(
                        fit:BoxFit.fitHeight,
                        child: Switch(
                          value: themeProvider.isDarkMode,
                          inactiveTrackColor: Colors.black38,
                          activeColor: Colors.white38,
                          onChanged: (value) {
                            themeProvider.toggleTheme(value);
                          },
                        ),
                      ),
                    ),
                  ],
                ) : IconButton(icon: Icon(Icons.dark_mode), onPressed:() => themeProvider.toggleTheme(!themeProvider.isDarkMode),);
              },
            ),
          ),
          // ViewerBody(),
          Expanded(
            child: AnimatedBuilder(
              animation: _controller,
              builder:(context, child) {
                switch (_controller.selectedIndex) {
                  case 0:
                    return ChatRoom();
                  case 1:
                    return EntranceCodeInputPage();
                  
                  default:
                    return ChatRoom();
                }
              },
            )
          ),
        ],
      ),
    );
  }
}

// class Pair<T1, T2> {
//   final T1 me;
//   final T2 msg;

//   Pair(this.me, this.msg);
// }


class ChatRoom extends StatefulWidget {
  @override
  _ChatRoomState createState() => _ChatRoomState();
}

class _ChatRoomState extends State<ChatRoom> {
  final TextEditingController _controller = TextEditingController();
  final FocusNode _focusNode = FocusNode();


  void _sendMessage(BuildContext context) {
    if (_controller.text.isNotEmpty) {
      Provider.of<InfoProvider>(context).addmessage(true, _controller.text);
      SendMessage(who: 'HIHI', contents: _controller.text).sendSignalToRust(null);
      _controller.clear();
      FocusScope.of(context).requestFocus(_focusNode);
    }
  }
  void _recvMessage(String _who, String _contents) {
    if (_contents.isNotEmpty) {
      Provider.of<InfoProvider>(context).addmessage(false, _contents);
    }
  }

  @override
  Widget build(BuildContext context) {
    final info = Provider.of<InfoProvider>(context);
    final messagesList = info.messagesList;

    return Scaffold(
      appBar: AppBar(
        title: Text('Chat Room'),
        automaticallyImplyLeading: false,
      ),
      body: StreamBuilder(
        stream: RecvMessage.rustSignalStream,
        builder: (context, snapshot) {
          final rustSignal = snapshot.data;
          if (snapshot.hasData && rustSignal != null) {
            _recvMessage("2", rustSignal.message.contents);
          }
          return ListView.builder(
            reverse: true,
            itemCount: messagesList.length,
            itemBuilder: (context, index) {
              return ListTile(
                title: Row(
                  mainAxisAlignment: messagesList[index].me ? MainAxisAlignment.end : MainAxisAlignment.start,
                  children: [
                    if (!messagesList[index].me) ...[
                      CircleAvatar(child: Text("A")), // 다른 사람의 메시지에는 아바타 표시
                      SizedBox(width: 10),
                    ],
                    Container(
                      padding: EdgeInsets.symmetric(horizontal: 15, vertical: 10),
                      margin: EdgeInsets.symmetric(vertical: 5, horizontal: 10),
                      decoration: BoxDecoration(
                        color: messagesList[index].me ? Colors.blue : Colors.grey[300],
                        borderRadius: BorderRadius.circular(20),
                      ),
                      child: Text(
                        messagesList[index].msg,
                        style: TextStyle(
                          color: messagesList[index].me ? Colors.white : Colors.black,
                        ),
                      ),
                    ),
                    if (messagesList[index].me) ...[
                      SizedBox(width: 10),
                      CircleAvatar(child: Text("Me")), // 사용자의 메시지에는 아바타 표시
                    ],
                  ],
                ),
              );
            },
          );
        }
      ),
      bottomNavigationBar: Padding(
        padding: const EdgeInsets.all(10.0),
        child: Row(
          children: <Widget>[
            Expanded(
              child: TextField(
                controller: _controller,
                focusNode: _focusNode,
                decoration: InputDecoration(
                  hintText: 'Enter a message...',
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(14),
                  ),
                ),
                onSubmitted: (value){
                  if (_controller.text.isNotEmpty) {
                    info.addmessage(true, _controller.text);
                    SendMessage(who: 'HIHI', contents: _controller.text).sendSignalToRust(null);
                    _controller.clear();
                    FocusScope.of(context).requestFocus(_focusNode);
                  }
                },
              ),
            ),
            IconButton(
              icon: Icon(Icons.send),
              onPressed: () {
               if (_controller.text.isNotEmpty) {
                  info.addmessage(true, _controller.text);
                  SendMessage(who: 'HIHI', contents: _controller.text).sendSignalToRust(null);
                  _controller.clear();
                  FocusScope.of(context).requestFocus(_focusNode);
                }
              },
            ),
          ],
        ),
      ),
    );
  }
}

class EntranceCodeInputPage extends StatefulWidget {
  @override
  _EntranceCodeInputPageState createState() => _EntranceCodeInputPageState();
}

class _EntranceCodeInputPageState extends State<EntranceCodeInputPage> {
  final TextEditingController _controllerCode = TextEditingController();
  final TextEditingController _controllerName = TextEditingController();

  void _enterChatroom() {
    if (_controllerCode.text.isNotEmpty) {
      setState(() {
        KnockIP(who: 'HIHI', password: _controllerCode.text).sendSignalToRust(null);
      });
    }
  }
  void _setMyname(BuildContext context) {
    final info = Provider.of<InfoProvider>(context);
    if (_controllerName.text.isNotEmpty) {
      info.setName(_controllerName.text);
      SetMyName(name: _controllerName.text).sendSignalToRust(null);
    }
  }

  @override
  Widget build(BuildContext context) {
    final info = Provider.of<InfoProvider>(context);
    _controllerName.text = info.name;

    return Scaffold(
      appBar: AppBar(
        title: Text('Enter Entrance Code'),
        automaticallyImplyLeading: false,
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            children: [
              Row(
                children: <Widget>[
                  Expanded(
                    child: TextField(
                      controller: _controllerName,
                      decoration: const InputDecoration(
                        border: OutlineInputBorder(),
                        hintText: 'Enter your Name',
                      ),
                      keyboardType: TextInputType.text,
                      onSubmitted: (value) {
                        SetMyName(name: _controllerName.text).sendSignalToRust(null);
                        info.setName(_controllerName.text);
                      },
                    ),
                  ),
                  IconButton(
                    icon: Icon(Icons.save),
                    onPressed: () {
                      SetMyName(name: _controllerName.text).sendSignalToRust(null);
                      info.setName(_controllerName.text);
                    },
                  ),
                ],
              ),
              Row(
                children: [
                  IconButton(
                    onPressed: (){ GetMyPassword().sendSignalToRust(null); },
                    icon: Icon(Icons.refresh)
                  ),
                  StreamBuilder(
                    stream: ThisisMyPassword.rustSignalStream,
                    builder: (context, snapshot) {
                      final rustSignal = snapshot.data;
                      if (snapshot.hasData && rustSignal != null) {
                        final pss = rustSignal.message.password;
                        return SelectableText(pss);
                      } 
                      return Text("← 초대코드");
                    }
                  ),
                ],
              ),
              StreamBuilder(
                stream: RecvMessage.rustSignalStream,
                builder: (context, snapshot) {
                  final rustSignal = snapshot.data;
                  if (snapshot.hasData && rustSignal != null) {
                    Provider.of<InfoProvider>(context).addmessage(false, rustSignal.message.contents);
                  }
                  return Text("");
                }
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class EntrancePage extends StatefulWidget {
  @override
  _EntrancePageState createState() => _EntrancePageState();
}

class _EntrancePageState extends State<EntrancePage> {
  TextEditingController nameController = TextEditingController();
  TextEditingController codeController = TextEditingController();

  @override
  Widget build(BuildContext context) {
    final info = Provider.of<InfoProvider>(context);
    nameController.text = info.name;

    return Scaffold(
      appBar: AppBar(
        title: Text('채팅방 입장'),
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              Container(
                constraints: BoxConstraints(maxWidth: 350),
                child: TextField(
                  controller: nameController,
                  decoration: InputDecoration(
                    labelText: '이름',
                    border: OutlineInputBorder(),
                  ),
                ),
              ),
              SizedBox(height: 20),
              Container(
                constraints: BoxConstraints(maxWidth: 350),
                child: TextField(
                  controller: codeController,
                  decoration: InputDecoration(
                    labelText: '입장 코드',
                    border: OutlineInputBorder(),
                  ),
                ),
              ),
              SizedBox(height: 20),
              ElevatedButton(
                onPressed: () {
                  KnockIP(who: 'HIHI', password: codeController.text).sendSignalToRust(null);
                  SetMyName(name: nameController.text).sendSignalToRust(null);
                  info.setName(nameController.text);
                  Navigator.pushNamed(context, '/MainPage');
                  setState(() { });
                },
                child: Text('입장'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  @override
  void dispose() {
    nameController.dispose();
    codeController.dispose();
    super.dispose();
  }
}
