import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:simple_icons/simple_icons.dart';
import './messages/generated.dart';
import './messages/chatmessage.pb.dart';

import 'package:sidebarx/sidebarx.dart';
import 'provider/themeprovider.dart';
import 'package:provider/provider.dart';
import 'dart:io';


void main() async {
  await initializeRust();
  runApp(
    MultiProvider(
      providers: [
        ChangeNotifierProvider( create: (context) => ThemeProvider(), ),
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
      home: Scaffold(
        body: Center(
          child: MainBody(),
        ),
      ),

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
    return Row(
      children: [
        Container(
          margin: EdgeInsets.fromLTRB(5, 5, 0, 5),
          child: SidebarX(
            controller: _controller,
            items: const [
              SidebarXItem(icon: Icons.person, label: ' Home'),
              SidebarXItem(icon: SimpleIcons.openai, label: ' RGBpalette'),
              SidebarXItem(icon: Icons.settings, label: ' Analyse'),
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
                  return DefChatRoom();
                case 1:
                  return ChatRoom();
                case 2:
                  return IPSettingsPage();
                
                default:
                  return DefChatRoom();
              }
            },
          )
        ),
      ],
    );
  }
}

class DefChatRoom extends StatefulWidget {
  const DefChatRoom({super.key});

  @override
  State<DefChatRoom> createState() => _DefChatRoomState();
}

class _DefChatRoomState extends State<DefChatRoom> {

  List<String> _chatlist = ["this", "is", "first", "chat"];

  @override
  Widget build(BuildContext context) {
    // return Row(
    //   children: [
    //     Expanded(
    //       child: ListView.builder(
    //         scrollDirection: Axis.horizontal,
    //         itemCount: _chatlist.length,
    //         itemBuilder:(context, index) => Text(_chatlist[index]),
    //       ),
    //     ),
    //     Container(
    //       margin: const EdgeInsets.all(20),
    //       padding: const EdgeInsets.all(20),
    //       height: 80,
    //       decoration: BoxDecoration(
    //         borderRadius: BorderRadius.circular(24.0),
    //         color: Colors.white,
    //       ),
    //       child: Center(
    //         child: TextField(),
    //       ),
    //     ),
    //   ],
    // );
    return Stack(
      alignment: Alignment.bottomCenter ,
      children: [
        Container(
          color: Colors.amberAccent,
          child: ListView(),
        ),
        Container(
          margin: const EdgeInsets.all(20),
          padding: const EdgeInsets.all(20),
          height: 80,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(24.0),
            color: Colors.white,
          ),
          child: Center(
            child: TextField(),
          ),
        ),
      ],
    );
  }
}

class Pair<T1, T2> {
  final T1 who;
  final T2 msg;

  Pair(this.who, this.msg);
}

class ChatRoom extends StatefulWidget {
  @override
  _ChatRoomState createState() => _ChatRoomState();
}

class _ChatRoomState extends State<ChatRoom> {
  // final List<String> messages = [];
  final List<Pair<int, String>> messagesList = [];
  final TextEditingController _controller = TextEditingController();
  final FocusNode _focusNode = FocusNode();


  void _sendMessage() {
    if (_controller.text.isNotEmpty) {
      setState(() {
        messagesList.insert(0, Pair(1, _controller.text));
        SendMessage(who: 'HIHI', contents: _controller.text).sendSignalToRust(null);
        _controller.clear();
        FocusScope.of(context).requestFocus(_focusNode);
      });
    }
  }
  void _recvMessage(String _who, String _contents) {
    if (_contents.isNotEmpty) {
      setState(() {
        messagesList.insert(0, Pair(2, _contents));
      });
    }
  }

  StreamBuilder _recvStream(){
    return StreamBuilder(
      stream: RecvMessage.rustSignalStream,
      builder: (context, snapshot) {
        final rustSignal = snapshot.data;
        if (rustSignal != null) {
          _recvMessage("2", rustSignal.message.contents);
        }
        return Text("");
      }
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Chat Room'),
      ),
      body: ListView.builder(
        reverse: true,
        itemCount: messagesList.length,
        itemBuilder: (context, index) {
          return ListTile(
            title: Text(
              messagesList[index].msg,
              textAlign: messagesList[index].who == 1 ? TextAlign.right : TextAlign.left,),
          );
        },
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
                onSubmitted: (value){_sendMessage();},
              ),
            ),
            IconButton(
              icon: Icon(Icons.send),
              onPressed: _sendMessage,
            ),
            // IconButton(
            //   icon: Icon(Icons.call_received),
            //   onPressed: _recvMessage,
            // ),
          ],
        ),
      ),
    );
  }
}

class IPSettingsPage extends StatefulWidget {
  @override
  _IPSettingsPageState createState() => _IPSettingsPageState();
}

class _IPSettingsPageState extends State<IPSettingsPage> {
  List<String> ipAddresses = [];
  final TextEditingController _ipController = TextEditingController();

  void _addIPAddress() {
    if (_ipController.text.isNotEmpty) {
      setState(() {
        ipAddresses.add(_ipController.text.trim());
        _ipController.clear();
      });
    }
  }

  void _removeIPAddress(String ip) {
    setState(() {
      ipAddresses.remove(ip);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('IP Settings'),
      ),
      body: Column(
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: TextField(
              controller: _ipController,
              decoration: InputDecoration(
                border: OutlineInputBorder(),
                labelText: 'Add IP Address',
                hintText: 'Enter valid IP address',
              ),
              keyboardType: TextInputType.number,
            ),
          ),
          ElevatedButton(
            onPressed: _addIPAddress,
            child: Text('Add IP Address'),
          ),
          Expanded(
            child: ListView.builder(
              itemCount: ipAddresses.length,
              itemBuilder: (context, index) {
                return ListTile(
                  title: Text(ipAddresses[index]),
                  trailing: IconButton(
                    icon: Icon(Icons.delete),
                    onPressed: () => _removeIPAddress(ipAddresses[index]),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}